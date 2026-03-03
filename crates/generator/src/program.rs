use arbitrary::{Arbitrary, Unstructured};

use oxiom_ir::boundary;
use oxiom_ir::css::*;
use oxiom_ir::font::*;
use oxiom_ir::html::*;
use oxiom_ir::js::*;

/// A complete fuzz program: font-face declarations, CSS rules, DOM tree, JS script, and keyframes.
#[derive(Debug, Clone)]
pub struct FuzzProgram {
    pub font_faces: Vec<FontFaceDecl>,
    pub css_rules: Vec<CssRule>,
    pub dom: DomTree,
    pub script: Vec<JsOperation>,
    pub keyframes: Vec<KeyframesRule>,
    pub at_rules: Vec<AtRule>,
}

impl<'a> Arbitrary<'a> for FuzzProgram {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let font_faces = generate_font_faces(u)?;
        let dom = generate_dom(u)?;
        let css_rules = generate_css_rules(u, &font_faces)?;
        let keyframes = generate_keyframes(u, &css_rules)?;
        let at_rules = generate_at_rules(u)?;

        // Generate mega-script: multi-phase state accumulation + timing chains
        let script = generate_mega_script(u, &font_faces)?;

        Ok(FuzzProgram {
            font_faces,
            css_rules,
            dom,
            script,
            keyframes,
            at_rules,
        })
    }
}

// ============================
// MEGA SCRIPT GENERATION
// ============================

/// Generate an extremely aggressive, state-accumulating script.
/// This is the core of crash-finding: create complex state, force layout,
/// mutate, force layout on stale refs, GC, access freed memory.
fn generate_mega_script(
    u: &mut Unstructured,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<JsOperation>> {
    // Choose between different mega-strategies
    let strategy: u8 = u.int_in_range(0..=9)?;

    let mut all_ops = match strategy {
        0 => generate_phased_attack(u, font_faces)?,
        1 => generate_timing_maze(u)?,
        2 => generate_chaos_storm(u)?,
        3 => generate_lifecycle_abuse(u, font_faces)?,
        4 => generate_observer_cascade(u)?,
        5 => generate_display_switching_storm(u)?,
        6 => generate_slot_redistribution(u)?,
        7 => generate_column_fragmentation(u)?,
        8 => generate_container_query_cycle(u)?,
        _ => generate_focus_navigation_attack(u)?,
    };

    // Always append a final GC + stale ref access burst
    append_stale_ref_burst(u, &mut all_ops)?;

    Ok(all_ops)
}

/// Phased attack: setup → layout → mutate → destroy → GC → stale access → rebuild.
/// This is the classic UAF-finding pattern with maximum state accumulation.
fn generate_phased_attack(
    u: &mut Unstructured,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::with_capacity(300);

    // ---- PHASE 1: Build complex state (15-30 ops) ----
    let setup_count = u.int_in_range(15..=30)?;
    for _ in 0..setup_count {
        match try_gen(u) {
            Some(op) => ops.push(op),
            None => break,
        }
    }

    // Attach shadow roots to several nodes
    for i in 0..u.int_in_range(2..=5)? {
        let target = NodeRef(i * 3);
        ops.push(JsOperation::AttachShadowRoot {
            target,
            mode: ShadowRootMode::Open,
        });
        ops.push(JsOperation::ShadowRootSetInnerHTML {
            host: target,
            html: InnerHtmlContent("<div><slot></slot><span style='display:grid'>shadow</span></div>".to_string()),
        });
    }

    // Set up containment and complex layout on nodes
    for i in 0..u.int_in_range(3..=8)? {
        let target = NodeRef(i * 2);
        let prop = pick_dangerous_css(u)?;
        ops.push(JsOperation::SetInlineStyle {
            target,
            property: prop,
        });
    }

    // Load fonts (triggers font swap → layout invalidation)
    for face in font_faces.iter().take(3) {
        ops.push(JsOperation::FontFaceLoad {
            family: String8(face.family.0.clone()),
        });
    }

    // ---- PHASE 2: Force initial layout everywhere (5-10 ops) ----
    for i in 0..u.int_in_range(5..=10)? {
        let target = NodeRef(i);
        match i % 4 {
            0 => ops.push(JsOperation::GetOffsetWidth { target }),
            1 => ops.push(JsOperation::GetOffsetHeight { target }),
            2 => ops.push(JsOperation::GetBoundingClientRect { target }),
            _ => ops.push(JsOperation::GetComputedStyle {
                target,
                property_name: StylePropertyName::Display,
            }),
        }
    }

    // ---- PHASE 3: Mutation storm (30-100 ops) ----
    // Generate as many mutations as bytes allow, interleaved with layout forcing
    let storm_count = u.int_in_range(30..=100)?;
    for j in 0..storm_count {
        match try_gen_mutation(u) {
            Some(op) => ops.push(op),
            None => break,
        }

        // Force layout every 3-5 operations (creates dangling pointer opportunities)
        if j % 4 == 0 {
            if let Some(target) = try_node(u) {
                ops.push(JsOperation::GetOffsetWidth { target });
            }
        }

        // GC every 15-20 ops
        if j % 17 == 0 {
            ops.push(JsOperation::ForceGC);
        }
    }

    // ---- PHASE 4: Teardown — remove nodes, destroy state ----
    let teardown_count = u.int_in_range(8..=15)?;
    for _ in 0..teardown_count {
        if let Some(target) = try_node(u) {
            let choice: u8 = u.int_in_range(0..=5).unwrap_or(0);
            match choice {
                0 => ops.push(JsOperation::RemoveChild { target }),
                1 => ops.push(JsOperation::SetInnerHTML {
                    target,
                    html: InnerHtmlContent("".to_string()),
                }),
                2 => ops.push(JsOperation::ReplaceChildren {
                    target,
                    html: InnerHtmlContent("".to_string()),
                }),
                3 => ops.push(JsOperation::AdoptNode { target }),
                4 => {
                    if let Some(start) = try_node(u) {
                        ops.push(JsOperation::RangeDeleteContents {
                            start_node: start,
                            end_node: target,
                        });
                    }
                }
                _ => ops.push(JsOperation::RemoveChild { target }),
            }
        }
    }

    // ---- PHASE 5: GC + stale reference access (the UAF trigger) ----
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::ForceGC);

    for i in 0..u.int_in_range(5..=12)? {
        let target = NodeRef(i);
        ops.push(JsOperation::GetOffsetWidth { target });
        ops.push(JsOperation::GetBoundingClientRect { target });
    }

    // ---- PHASE 6: Rebuild via innerHTML + layout (double-free territory) ----
    for i in 0..u.int_in_range(3..=6)? {
        let target = NodeRef(i * 4);
        ops.push(JsOperation::SetInnerHTML {
            target,
            html: InnerHtmlContent(
                "<div style='display:grid;contain:strict'><span>rebuilt</span></div>".to_string(),
            ),
        });
        ops.push(JsOperation::GetOffsetWidth { target });
    }
    ops.push(JsOperation::ForceGC);

    // Wrap phases 3-5 in timing chain for temporal sensitivity
    ops = wrap_in_timing_chain(u, ops)?;

    Ok(ops)
}

/// Timing maze: 5-8 levels of nested timing (rAF → microtask → rAF → setTimeout → rAF).
/// Each level mutates different nodes and forces layout. The timing interleaving
/// creates windows where freed memory can be reused.
fn generate_timing_maze(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let depth = u.int_in_range(5..=8)?;
    let mut ops = Vec::with_capacity(100);

    // Initial setup
    for _ in 0..u.int_in_range(5..=10)? {
        match try_gen(u) {
            Some(op) => ops.push(op),
            None => break,
        }
    }

    // Force initial layout
    for i in 0..5u16 {
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(i) });
    }

    // Build timing chain from inside out
    let mut inner: Vec<JsOperation> = vec![
        JsOperation::ForceGC,
        JsOperation::ForceGC,
    ];
    // Innermost: access stale refs
    for i in 0..8u16 {
        inner.push(JsOperation::GetOffsetWidth { target: NodeRef(i) });
        inner.push(JsOperation::GetBoundingClientRect { target: NodeRef(i) });
    }

    for level in 0..depth {
        let mut frame_ops: Vec<JsOperation> = Vec::new();

        // Each level: mutate → layout → GC → stale access
        let target = NodeRef((level * 3) as u16);
        let prop = pick_dangerous_css(u)?;
        frame_ops.push(JsOperation::SetInlineStyle {
            target,
            property: prop,
        });
        frame_ops.push(JsOperation::GetOffsetWidth { target });

        // Destroy something at this level
        let destroy_target = NodeRef((level * 3 + 1) as u16);
        frame_ops.push(JsOperation::RemoveChild { target: destroy_target });
        frame_ops.push(JsOperation::ForceGC);
        frame_ops.push(JsOperation::GetOffsetWidth { target: destroy_target }); // stale!

        // Add some random mutations
        for _ in 0..u.int_in_range(2..=6)? {
            match try_gen_mutation(u) {
                Some(op) => frame_ops.push(op),
                None => break,
            }
        }

        // Nest the inner chain using alternating timing APIs
        let wrapped_inner = match level % 3 {
            0 => JsOperation::RequestAnimationFrame { operations: inner },
            1 => JsOperation::QueueMicrotask { operations: inner },
            _ => JsOperation::SetTimeout { delay_ms: 0, operations: inner },
        };
        frame_ops.push(wrapped_inner);

        inner = frame_ops;
    }

    // Outermost wrapper
    ops.push(JsOperation::RequestAnimationFrame { operations: inner });

    // Also run parallel timing chains (concurrent stale access)
    let mut parallel_chain: Vec<JsOperation> = Vec::new();
    for _ in 0..u.int_in_range(5..=12)? {
        match try_gen_mutation(u) {
            Some(op) => parallel_chain.push(op),
            None => break,
        }
    }
    parallel_chain.push(JsOperation::ForceGC);
    for i in 0..6u16 {
        parallel_chain.push(JsOperation::GetOffsetWidth { target: NodeRef(i * 5) });
    }
    ops.push(JsOperation::SetTimeout {
        delay_ms: 0,
        operations: parallel_chain,
    });

    Ok(ops)
}

/// Chaos storm: 80-300 rapid-fire random ops with interleaved GC + layout forcing.
/// Pure volume — overwhelm Chrome's lifecycle management.
fn generate_chaos_storm(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::with_capacity(400);

    // Generate as many ops as the byte buffer allows
    let max_ops = u.int_in_range(80..=300)?;
    for j in 0..max_ops {
        match try_gen(u) {
            Some(op) => ops.push(op),
            None => break,
        }

        // Aggressive layout forcing every 3 ops
        if j % 3 == 0 {
            if let Some(target) = try_node(u) {
                ops.push(JsOperation::GetOffsetWidth { target });
            }
        }

        // GC every 10 ops
        if j % 10 == 0 {
            ops.push(JsOperation::ForceGC);
        }

        // Scroll into view every 15 ops (different layout path)
        if j % 15 == 0 {
            if let Some(target) = try_node(u) {
                ops.push(JsOperation::ScrollIntoView { target });
            }
        }
    }

    // Final GC + massive stale ref scan
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::ForceGC);
    for i in 0..20u16 {
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(i) });
        ops.push(JsOperation::GetBoundingClientRect { target: NodeRef(i) });
    }

    // Wrap half the ops in a timing chain
    ops = wrap_in_timing_chain(u, ops)?;

    Ok(ops)
}

/// Lifecycle abuse: create → layout → adoptNode → layout stale → re-insert → iframe cycle.
/// Targets document lifecycle and node adoption edge cases.
fn generate_lifecycle_abuse(
    u: &mut Unstructured,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::with_capacity(200);

    let cycles = u.int_in_range(3..=6)?;
    for cycle in 0..cycles {
        let base = (cycle * 8) as u16;

        // Create complex subtree
        let parent = NodeRef(base);
        for j in 0..u.int_in_range(2..=4)? {
            ops.push(JsOperation::CreateElement {
                parent,
                tag: u.arbitrary().unwrap_or(CreateElementTag::Div),
            });
            let child = NodeRef(base + j as u16 + 1);
            ops.push(JsOperation::SetInlineStyle {
                target: child,
                property: pick_dangerous_css(u)?,
            });
        }

        // Shadow DOM on parent
        ops.push(JsOperation::AttachShadowRoot {
            target: parent,
            mode: ShadowRootMode::Open,
        });
        ops.push(JsOperation::ShadowRootSetInnerHTML {
            host: parent,
            html: InnerHtmlContent(
                "<slot></slot><div style='display:contents'>shadow</div>".to_string(),
            ),
        });

        // Force layout
        ops.push(JsOperation::GetOffsetWidth { target: parent });
        ops.push(JsOperation::GetBoundingClientRect { target: parent });

        // adoptNode — removes from document, layout objects become stale
        ops.push(JsOperation::AdoptNode { target: parent });
        ops.push(JsOperation::ForceGC);

        // Access stale layout (UAF territory)
        ops.push(JsOperation::GetOffsetWidth { target: parent });
        ops.push(JsOperation::GetOffsetHeight { target: parent });

        // Re-insert into document
        let new_parent = NodeRef(base.wrapping_add(10));
        ops.push(JsOperation::AppendChild {
            parent: new_parent,
            child: parent,
        });
        ops.push(JsOperation::GetOffsetWidth { target: parent });

        // Iframe cycle
        ops.push(JsOperation::CreateIframe {
            target: parent,
            src_html: InnerHtmlContent(
                "<html><body style='display:grid'><div>iframe content</div></body></html>"
                    .to_string(),
            ),
        });
        ops.push(JsOperation::GetOffsetWidth { target: parent });
        ops.push(JsOperation::RemoveIframe { target: parent });
        ops.push(JsOperation::ForceGC);
        ops.push(JsOperation::GetOffsetWidth { target: parent });
    }

    // Font swap stress (font load → layout invalidation → stale sizing)
    for face in font_faces.iter().take(3) {
        ops.push(JsOperation::FontFaceAdd {
            family: String8(face.family.0.clone()),
            source: FontFaceSource::DataUrl,
        });
        ops.push(JsOperation::FontFaceLoad {
            family: String8(face.family.0.clone()),
        });
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(0) });
        ops.push(JsOperation::FontFaceRemove {
            family: String8(face.family.0.clone()),
        });
        ops.push(JsOperation::ForceGC);
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(0) });
    }

    // Editing stress at the end
    ops.push(JsOperation::SetAttribute {
        target: NodeRef(0),
        attr_name: AttrName::ContentEditable,
        attr_value: String8("true".to_string()),
    });
    ops.push(JsOperation::FocusNode { target: NodeRef(0) });
    ops.push(JsOperation::SelectAllContent);
    for _ in 0..u.int_in_range(3..=8)? {
        let cmd: ExecCommandType = u.arbitrary().unwrap_or(ExecCommandType::Delete);
        ops.push(JsOperation::ExecCommand { command: cmd });
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(0) });
    }
    ops.push(JsOperation::ForceGC);

    ops = wrap_in_timing_chain(u, ops)?;

    Ok(ops)
}

/// Observer cascade: set up MutationObserver + ResizeObserver + IntersectionObserver,
/// then trigger mutations that cause cascading callbacks with layout forcing.
fn generate_observer_cascade(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::with_capacity(200);

    // Setup: create complex layout state
    for i in 0..u.int_in_range(5..=10)? {
        let target = NodeRef(i as u16);
        ops.push(JsOperation::SetInlineStyle {
            target,
            property: pick_dangerous_css(u)?,
        });
    }

    // Force initial layout
    for i in 0..5u16 {
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(i) });
    }

    // Set up multiple observers with callbacks that mutate the DOM
    for i in 0..u.int_in_range(2..=4)? {
        let target = NodeRef(i as u16 * 3);
        let mutate_target = NodeRef(i as u16 * 3 + 1);

        // MutationObserver: observe and mutate in callback
        ops.push(JsOperation::ObserveAndMutate {
            observe_target: target,
            mutate_target,
            mutation_op: u.arbitrary().unwrap_or(MutationOp::SetInnerHTML),
        });

        // ResizeObserver: force layout in callback (re-entrant layout)
        ops.push(JsOperation::ResizeObserverObserve {
            target,
            callback_ops: vec![
                JsOperation::SetInlineStyle {
                    target: mutate_target,
                    property: CssProperty::Width(LengthOrAuto::Length(LengthValue::Px(
                        (i * 100 + 50) as i32,
                    ))),
                },
                JsOperation::GetOffsetWidth { target: mutate_target },
                JsOperation::ForceGC,
            ],
        });

        // IntersectionObserver: remove nodes in callback
        ops.push(JsOperation::IntersectionObserverObserve {
            target,
            callback_ops: vec![
                JsOperation::RemoveChild { target: mutate_target },
                JsOperation::GetOffsetWidth { target: mutate_target },
            ],
        });
    }

    // Now trigger all the observers by mutating the observed nodes
    for i in 0..u.int_in_range(10..=30)? {
        let target = NodeRef((i % 12) as u16);
        match try_gen_mutation(u) {
            Some(op) => ops.push(op),
            None => break,
        }
        ops.push(JsOperation::GetOffsetWidth { target });

        if i % 5 == 0 {
            ops.push(JsOperation::ForceGC);
        }
    }

    // ScrollIntoView to trigger IntersectionObserver callbacks
    for i in 0..u.int_in_range(3..=6)? {
        let target = NodeRef(i as u16);
        ops.push(JsOperation::ScrollIntoView { target });
        ops.push(JsOperation::GetOffsetWidth { target });
    }

    // Resize elements to trigger ResizeObserver callbacks
    for i in 0..u.int_in_range(3..=6)? {
        let target = NodeRef(i as u16 * 3);
        ops.push(JsOperation::SetInlineStyle {
            target,
            property: CssProperty::Width(LengthOrAuto::Length(LengthValue::Px(
                (i * 200 + 1) as i32,
            ))),
        });
        ops.push(JsOperation::GetOffsetWidth { target });
    }

    // Final GC + stale access
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::ForceGC);
    for i in 0..15u16 {
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(i) });
    }

    ops = wrap_in_timing_chain(u, ops)?;

    Ok(ops)
}

/// Display switching storm: rapidly switch display between layout-creating values
/// (grid/flex/table) and destructive values (none/contents), forcing layout between
/// each switch. This is the #1 UAF pattern for Blink layout bugs (~40% of layout crashes).
fn generate_display_switching_storm(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();
    let display_values = [
        DisplayValue::Grid,
        DisplayValue::Flex,
        DisplayValue::Table,
        DisplayValue::InlineGrid,
        DisplayValue::InlineFlex,
        DisplayValue::Block,
    ];
    let destructive_values = [DisplayValue::None, DisplayValue::Contents, DisplayValue::Inline];
    let rounds = u.int_in_range(5..=15)?;

    for _ in 0..rounds {
        let node = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
        // Set initial display (layout-creating)
        let initial_idx: usize = u.arbitrary()?;
        let initial_display = display_values[initial_idx % display_values.len()];
        ops.push(JsOperation::SetInlineStyle {
            target: node,
            property: CssProperty::Display(initial_display),
        });
        // Force layout
        ops.push(JsOperation::GetOffsetWidth { target: node });

        // Switch to destructive display
        let destructive_idx: usize = u.arbitrary()?;
        let dest_display = destructive_values[destructive_idx % destructive_values.len()];
        ops.push(JsOperation::SetInlineStyle {
            target: node,
            property: CssProperty::Display(dest_display),
        });
        // Force layout on stale LayoutObject
        ops.push(JsOperation::GetOffsetWidth { target: node });
        ops.push(JsOperation::GetBoundingClientRect { target: node });

        // Add content-visibility switching
        if u.arbitrary::<bool>()? {
            ops.push(JsOperation::SetInlineStyle {
                target: node,
                property: CssProperty::ContentVisibility(ContentVisibilityValue::Hidden),
            });
            ops.push(JsOperation::GetOffsetHeight { target: node });
            ops.push(JsOperation::SetInlineStyle {
                target: node,
                property: CssProperty::ContentVisibility(ContentVisibilityValue::Auto),
            });
        }

        // GC to free stale objects
        if u.arbitrary::<bool>()? {
            ops.push(JsOperation::ForceGC);
        }

        // Switch back to creating display (access freed memory)
        ops.push(JsOperation::SetInlineStyle {
            target: node,
            property: CssProperty::Display(DisplayValue::Grid),
        });
        ops.push(JsOperation::GetOffsetWidth { target: node });
    }
    Ok(ops)
}

/// Shadow DOM slot redistribution: attach shadow roots with multiple slots,
/// then rapidly reassign slot attributes on light DOM children. This triggers
/// slot redistribution logic which is prone to stale reference bugs.
fn generate_slot_redistribution(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();
    // Attach shadow roots to several hosts
    let host_count = u.int_in_range(2..=5)?;
    for _ in 0..host_count {
        let host = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
        ops.push(JsOperation::AttachShadowRoot {
            target: host,
            mode: ShadowRootMode::Open,
        });
        ops.push(JsOperation::ShadowRootSetInnerHTML {
            host,
            html: InnerHtmlContent(
                "<div><slot></slot><slot name='a'></slot><slot name='b'></slot></div>".to_string(),
            ),
        });
    }
    // Force layout
    let layout_target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
    ops.push(JsOperation::GetOffsetWidth {
        target: layout_target,
    });

    // Reassign slots rapidly
    let slot_names = ["", "a", "b", "c"];
    let reassign_rounds = u.int_in_range(5..=20)?;
    for _ in 0..reassign_rounds {
        let target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
        let name_idx: usize = u.arbitrary()?;
        ops.push(JsOperation::SetSlotAttribute {
            target,
            slot_name: String8(slot_names[name_idx % slot_names.len()].to_string()),
        });
        // Force layout between reassignments
        if u.arbitrary::<bool>()? {
            let lt = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
            ops.push(JsOperation::GetOffsetWidth { target: lt });
        }
    }
    // GC + layout
    ops.push(JsOperation::ForceGC);
    let final_target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
    ops.push(JsOperation::GetOffsetWidth {
        target: final_target,
    });
    Ok(ops)
}

/// Multi-column fragmentation: set up a multi-column container, rapidly change
/// column count while inserting/removing children and forcing layout. Then switch
/// to display:none and back. Column fragmentation logic is a rich source of crashes.
fn generate_column_fragmentation(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();
    let target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;

    // Set up multi-column container
    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::ColumnCount(ColumnCountValue::Number(u.int_in_range(2..=10)?)),
    });
    ops.push(JsOperation::GetOffsetWidth { target });

    // Rapidly change column count
    let rounds = u.int_in_range(5..=15)?;
    for _ in 0..rounds {
        let count = u.int_in_range(1..=20)?;
        ops.push(JsOperation::SetInlineStyle {
            target,
            property: CssProperty::ColumnCount(ColumnCountValue::Number(count)),
        });
        // Insert/remove children
        if u.arbitrary::<bool>()? {
            ops.push(JsOperation::SetInnerHTML {
                target,
                html: InnerHtmlContent(
                    "<p>col1</p><p style='break-before:column'>col2</p><p>col3</p>".to_string(),
                ),
            });
        }
        if u.arbitrary::<bool>()? {
            if let Some(child) = try_node(u) {
                ops.push(JsOperation::RemoveChild { target: child });
            }
        }
        ops.push(JsOperation::GetOffsetWidth { target });
    }
    // Switch to display:none then back
    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::Display(DisplayValue::None),
    });
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::Display(DisplayValue::Block),
    });
    ops.push(JsOperation::GetOffsetWidth { target });
    Ok(ops)
}

/// Container query cycle: set up a container query container, then rapidly toggle
/// container-type and resize, triggering @container evaluation on corrupted state.
/// Can create dependency cycles in container query resolution.
fn generate_container_query_cycle(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();
    let target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;

    // Set up container query container
    ops.push(JsOperation::SetContainerType {
        target,
        container_type: oxiom_ir::js::ContainerTypeValue::InlineSize,
    });
    ops.push(JsOperation::GetOffsetWidth { target });

    // Rapidly toggle container-type (can corrupt container query state)
    let types = [
        oxiom_ir::js::ContainerTypeValue::Normal,
        oxiom_ir::js::ContainerTypeValue::InlineSize,
        oxiom_ir::js::ContainerTypeValue::Size,
    ];
    let rounds = u.int_in_range(5..=15)?;
    for _ in 0..rounds {
        let idx: usize = u.arbitrary()?;
        ops.push(JsOperation::SetContainerType {
            target,
            container_type: types[idx % types.len()],
        });
        // Change size to trigger @container
        ops.push(JsOperation::SetInlineStyle {
            target,
            property: CssProperty::Width(LengthOrAuto::Length(LengthValue::Px(
                u.int_in_range(1..=1000)?,
            ))),
        });
        ops.push(JsOperation::GetOffsetWidth { target });
        // Also switch display to invalidate container
        if u.arbitrary::<bool>()? {
            ops.push(JsOperation::SetInlineStyle {
                target,
                property: CssProperty::Display(DisplayValue::None),
            });
            ops.push(JsOperation::GetOffsetWidth { target });
            ops.push(JsOperation::SetInlineStyle {
                target,
                property: CssProperty::Display(DisplayValue::Block),
            });
        }
    }
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::GetOffsetWidth { target });
    Ok(ops)
}

/// Focus/selection + DOM mutation UAF: make nodes contenteditable, create selections
/// across nodes, remove selected nodes, then execute editing commands on stale selections.
/// Targets the editing/selection lifecycle which is a rich source of UAF bugs.
fn generate_focus_navigation_attack(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    // Make nodes contenteditable
    let edit_count = u.int_in_range(2..=5)?;
    for _ in 0..edit_count {
        let target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
        ops.push(JsOperation::SetAttribute {
            target,
            attr_name: AttrName::ContentEditable,
            attr_value: String8("true".to_string()),
        });
    }

    // Focus + create selection
    let focus_target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
    ops.push(JsOperation::FocusNode {
        target: focus_target,
    });
    ops.push(JsOperation::SelectAllContent);

    // Set selection range
    ops.push(JsOperation::SetSelectionRange {
        anchor_node: try_node(u).ok_or(arbitrary::Error::NotEnoughData)?,
        anchor_offset: u.int_in_range(0..=5)?,
        focus_node: try_node(u).ok_or(arbitrary::Error::NotEnoughData)?,
        focus_offset: u.int_in_range(0..=5)?,
    });

    // Remove nodes that are in the selection
    let remove_count = u.int_in_range(1..=4)?;
    for _ in 0..remove_count {
        let target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
        ops.push(JsOperation::RemoveChild { target });
    }

    // Execute commands on stale selection
    let commands = [
        ExecCommandType::InsertHTML,
        ExecCommandType::Delete,
        ExecCommandType::Bold,
        ExecCommandType::InsertText,
        ExecCommandType::InsertUnorderedList,
        ExecCommandType::FormatBlock,
    ];
    let cmd_count = u.int_in_range(2..=6)?;
    for _ in 0..cmd_count {
        let idx: usize = u.arbitrary()?;
        ops.push(JsOperation::ExecCommand {
            command: commands[idx % commands.len()],
        });
        let layout_target = try_node(u).ok_or(arbitrary::Error::NotEnoughData)?;
        ops.push(JsOperation::GetOffsetWidth {
            target: layout_target,
        });
    }

    // GC + access selection
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::SelectAllContent);
    ops.push(JsOperation::GetOffsetWidth {
        target: focus_target,
    });
    Ok(ops)
}

// ============================
// HELPER FUNCTIONS
// ============================

/// Pick CSS properties that are most likely to trigger layout bugs.
fn pick_dangerous_css(u: &mut Unstructured) -> arbitrary::Result<CssProperty> {
    let choice: u8 = u.int_in_range(0..=19)?;
    Ok(match choice {
        0 => CssProperty::Display(DisplayValue::None),
        1 => CssProperty::Display(DisplayValue::Contents),
        2 => CssProperty::Display(DisplayValue::Grid),
        3 => CssProperty::Display(DisplayValue::Flex),
        4 => CssProperty::Display(DisplayValue::Table),
        5 => CssProperty::Display(DisplayValue::InlineGrid),
        6 => CssProperty::ContentVisibility(ContentVisibilityValue::Hidden),
        7 => CssProperty::ContentVisibility(ContentVisibilityValue::Auto),
        8 => CssProperty::Contain(ContainValue::Strict),
        9 => CssProperty::Contain(ContainValue::Content),
        10 => CssProperty::Position(PositionValue::Fixed),
        11 => CssProperty::Position(PositionValue::Absolute),
        12 => CssProperty::Position(PositionValue::Sticky),
        13 => CssProperty::Overflow(OverflowValue::Hidden),
        14 => CssProperty::Visibility(VisibilityValue::Collapse),
        15 => CssProperty::Width(LengthOrAuto::Length(LengthValue::Zero)),
        16 => CssProperty::Height(LengthOrAuto::Length(LengthValue::Zero)),
        17 => CssProperty::Opacity(OpacityValue(0)),
        18 => CssProperty::Transform(TransformList {
            transforms: vec![TransformFunction::Scale(0, 0)],
        }),
        _ => CssProperty::ColumnCount(ColumnCountValue::Number(u.int_in_range(1..=5)?)),
    })
}

/// Try to generate an arbitrary JsOperation, returning None if bytes exhausted.
fn try_gen(u: &mut Unstructured) -> Option<JsOperation> {
    u.arbitrary().ok()
}

/// Try to generate a mutation-focused JsOperation.
fn try_gen_mutation(u: &mut Unstructured) -> Option<JsOperation> {
    let choice: u8 = u.int_in_range(0..=18).ok()?;
    let target = try_node(u)?;
    Some(match choice {
        0 => JsOperation::RemoveChild { target },
        1 => {
            let parent = try_node(u)?;
            JsOperation::AppendChild {
                parent,
                child: target,
            }
        }
        2 => JsOperation::SetInnerHTML {
            target,
            html: u.arbitrary().ok()?,
        },
        3 => JsOperation::SetInlineStyle {
            target,
            property: pick_dangerous_css(u).ok()?,
        },
        4 => JsOperation::CloneNode {
            source: target,
            deep: true,
            append_to: try_node(u)?,
        },
        5 => JsOperation::AdoptNode { target },
        6 => JsOperation::InsertAdjacentHTML {
            target,
            position: u.arbitrary().ok()?,
            html: u.arbitrary().ok()?,
        },
        7 => {
            let start = try_node(u)?;
            JsOperation::RangeDeleteContents {
                start_node: start,
                end_node: target,
            }
        }
        8 => JsOperation::ReplaceWith {
            target,
            html: u.arbitrary().ok()?,
        },
        9 => JsOperation::SetTextContent {
            target,
            text: u.arbitrary().ok()?,
        },
        10 => {
            let parent = try_node(u)?;
            let ref_child = try_node(u)?;
            JsOperation::InsertBefore {
                parent,
                new_child: target,
                ref_child,
            }
        }
        11 => JsOperation::ReplaceChildren {
            target,
            html: u.arbitrary().ok()?,
        },
        12 => JsOperation::Normalize { target },
        13 => JsOperation::CreateElement {
            parent: target,
            tag: u.arbitrary().ok()?,
        },
        14 => JsOperation::ToggleClass {
            target,
            class_name: u.arbitrary().ok()?,
        },
        15 => JsOperation::SetAttribute {
            target,
            attr_name: u.arbitrary().ok()?,
            attr_value: u.arbitrary().ok()?,
        },
        16 => JsOperation::RemoveAttribute {
            target,
            attr_name: u.arbitrary().ok()?,
        },
        17 => JsOperation::ShadowRootSetInnerHTML {
            host: target,
            html: u.arbitrary().ok()?,
        },
        _ => JsOperation::ScrollIntoView { target },
    })
}

/// Try to generate a NodeRef.
fn try_node(u: &mut Unstructured) -> Option<NodeRef> {
    u.arbitrary().ok()
}

/// Append a burst of stale reference accesses after GC.
fn append_stale_ref_burst(
    u: &mut Unstructured,
    ops: &mut Vec<JsOperation>,
) -> arbitrary::Result<()> {
    // Double GC to ensure memory is freed
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::ForceGC);

    // Access many nodes — some may have been freed
    let count = u.int_in_range(10..=25).unwrap_or(15);
    for i in 0..count as u16 {
        ops.push(JsOperation::GetOffsetWidth { target: NodeRef(i) });
        if i % 3 == 0 {
            ops.push(JsOperation::GetBoundingClientRect { target: NodeRef(i) });
        }
        if i % 5 == 0 {
            ops.push(JsOperation::GetScrollMetrics { target: NodeRef(i) });
        }
        if i % 7 == 0 {
            ops.push(JsOperation::TreeWalkerTraverse { root: NodeRef(i) });
        }
    }

    Ok(())
}

/// Wrap operations in a timing chain: split into chunks, nest in rAF/microtask/setTimeout.
fn wrap_in_timing_chain(
    _u: &mut Unstructured,
    ops: Vec<JsOperation>,
) -> arbitrary::Result<Vec<JsOperation>> {
    if ops.len() < 20 {
        return Ok(ops);
    }

    // Split ops into 4 chunks
    let chunk_size = ops.len() / 4;
    let mut chunks: Vec<Vec<JsOperation>> = Vec::new();
    for chunk in ops.chunks(chunk_size.max(1)) {
        chunks.push(chunk.to_vec());
    }

    let mut result = Vec::new();

    // Chunk 0: runs immediately
    if let Some(chunk) = chunks.get(0) {
        result.extend_from_slice(chunk);
    }

    // Chunk 1 in rAF
    if let Some(chunk1) = chunks.get(1) {
        let mut raf_ops = chunk1.clone();
        raf_ops.push(JsOperation::ForceGC);

        // Chunk 2 in nested rAF inside rAF
        if let Some(chunk2) = chunks.get(2) {
            let mut inner_raf = chunk2.clone();

            // Chunk 3 in microtask inside nested rAF
            if let Some(chunk3) = chunks.get(3) {
                let mut micro_ops = chunk3.clone();
                // Final stale access inside microtask
                for i in 0..5u16 {
                    micro_ops.push(JsOperation::GetOffsetWidth { target: NodeRef(i) });
                }
                micro_ops.push(JsOperation::ForceGC);
                inner_raf.push(JsOperation::QueueMicrotask {
                    operations: micro_ops,
                });
            }

            raf_ops.push(JsOperation::RequestAnimationFrame {
                operations: inner_raf,
            });
        }

        result.push(JsOperation::RequestAnimationFrame {
            operations: raf_ops,
        });
    }

    // Also launch a parallel setTimeout chain for concurrent access
    let mut timeout_ops = Vec::new();
    for i in 0..8u16 {
        timeout_ops.push(JsOperation::GetOffsetWidth { target: NodeRef(i * 2) });
    }
    timeout_ops.push(JsOperation::ForceGC);
    for i in 0..8u16 {
        timeout_ops.push(JsOperation::GetBoundingClientRect { target: NodeRef(i * 2 + 1) });
    }
    result.push(JsOperation::SetTimeout {
        delay_ms: 0,
        operations: timeout_ops,
    });

    Ok(result)
}

// ============================
// DOM, CSS, KEYFRAMES, AT-RULES GENERATION
// ============================

fn generate_font_faces(u: &mut Unstructured) -> arbitrary::Result<Vec<FontFaceDecl>> {
    let count = u.int_in_range(1..=5)?;
    let mut faces = Vec::with_capacity(count);
    for _ in 0..count {
        faces.push(u.arbitrary()?);
    }
    Ok(faces)
}

/// Generate a large DOM tree for maximum crash surface.
fn generate_dom(u: &mut Unstructured) -> arbitrary::Result<DomTree> {
    let mode: u8 = u.int_in_range(0..=9)?;

    let (root_count, max_depth, max_children) = match mode {
        0 => {
            // Wide tree: 30-60 root children
            let count = u.int_in_range(30..=60)?;
            (count, 2usize, 2usize)
        }
        1 => {
            // Deep tree: depth 8, narrow
            let count = u.int_in_range(1..=3)?;
            (count, 8usize, 2usize)
        }
        _ => {
            // Normal large: 8-25 root children, depth 5-6
            let count = u.int_in_range(8..=25)?;
            (count, 5usize, 4usize)
        }
    };

    let mut children = Vec::with_capacity(root_count);
    for _ in 0..root_count {
        children.push(generate_dom_node(u, 0, max_depth, max_children)?);
    }
    Ok(DomTree {
        root_children: children,
    })
}

fn generate_dom_node(
    u: &mut Unstructured,
    depth: usize,
    max_depth: usize,
    max_children: usize,
) -> arbitrary::Result<DomNode> {
    let element: HtmlElement = u.arbitrary()?;
    let text_content = if u.ratio(1, 3)? {
        Some(u.arbitrary()?)
    } else {
        None
    };

    let children = if depth < max_depth && !element.is_void() {
        let child_count = u.int_in_range(0..=max_children)?;
        let mut kids = Vec::with_capacity(child_count);
        for _ in 0..child_count {
            kids.push(generate_dom_node(u, depth + 1, max_depth, max_children)?);
        }
        kids
    } else {
        vec![]
    };

    Ok(DomNode {
        element,
        text_content,
        children,
    })
}

fn generate_css_rules(
    u: &mut Unstructured,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<CssRule>> {
    let count = u.int_in_range(5..=15)?;
    let mut rules = Vec::with_capacity(count);

    for _ in 0..count {
        let selector: Selector = u.arbitrary()?;
        let decl_count = u.int_in_range(2..=7)?;
        let mut declarations = Vec::with_capacity(decl_count);

        for _ in 0..decl_count {
            let property = if u.ratio(1, 3)? {
                generate_boundary_property(u)?
            } else {
                u.arbitrary()?
            };

            let property = if !font_faces.is_empty() && u.ratio(1, 4)? {
                let face_idx = u.int_in_range(0..=font_faces.len() - 1)?;
                CssProperty::FontFamily(FontFamilyValue::Named(String8(
                    font_faces[face_idx].family.0.clone(),
                )))
            } else {
                property
            };

            declarations.push(CssDeclaration {
                property,
                important: u.ratio(1, 5)?,
            });
        }

        rules.push(CssRule {
            selector,
            declarations,
        });
    }

    Ok(rules)
}

fn generate_boundary_property(u: &mut Unstructured) -> arbitrary::Result<CssProperty> {
    let choice: u8 = u.int_in_range(0..=15)?;
    Ok(match choice {
        0 => {
            let v = boundary::boundary_font_sizes();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::FontSize(v[i].clone())
        }
        1 => {
            let v = boundary::boundary_displays();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::Display(v[i])
        }
        2 => {
            let v = boundary::boundary_positions();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::Position(v[i])
        }
        3 => {
            let v = boundary::boundary_content_visibilities();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::ContentVisibility(v[i])
        }
        4 => {
            let v = boundary::boundary_contains();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::Contain(v[i])
        }
        5 => {
            let v = boundary::boundary_font_weights();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::FontWeight(v[i].clone())
        }
        6 => {
            let v = boundary::boundary_z_indices();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::ZIndex(v[i].clone())
        }
        7 => {
            let v = boundary::boundary_dimensions();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::Width(v[i].clone())
        }
        8 => {
            let v = boundary::boundary_opacities();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::Opacity(v[i].clone())
        }
        9 => {
            let v = boundary::boundary_will_changes();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::WillChange(v[i])
        }
        10 => {
            let v = boundary::boundary_border_widths();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::BorderWidth(v[i].clone())
        }
        11 => {
            let v = boundary::boundary_border_radii();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::BorderRadius(v[i].clone())
        }
        12 => {
            let v = boundary::boundary_colors();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::Color(v[i].clone())
        }
        13 => {
            let v = boundary::boundary_colors();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::BackgroundColor(v[i].clone())
        }
        14 => {
            let v = boundary::boundary_border_styles();
            let i = u.int_in_range(0..=v.len() - 1)?;
            CssProperty::BorderStyle(v[i])
        }
        _ => {
            let v = boundary::boundary_global_keywords();
            let i = u.int_in_range(0..=v.len() - 1)?;
            let prop: CssPropertyName = u.arbitrary()?;
            CssProperty::GlobalReset(prop, v[i])
        }
    })
}

fn generate_keyframes(
    u: &mut Unstructured,
    css_rules: &[CssRule],
) -> arbitrary::Result<Vec<KeyframesRule>> {
    let mut keyframes = Vec::new();

    for rule in css_rules {
        for decl in &rule.declarations {
            if let CssProperty::Animation(anim) = &decl.property {
                let stop_count = u.int_in_range(2..=5)?;
                let mut stops = Vec::with_capacity(stop_count);

                for j in 0..stop_count {
                    let offset = if j == 0 {
                        KeyframeOffset::From
                    } else if j == stop_count - 1 {
                        KeyframeOffset::To
                    } else {
                        KeyframeOffset::Percent(
                            ((j as u16 * 100) / (stop_count as u16 - 1)).min(100) as u8,
                        )
                    };

                    let decl_count = u.int_in_range(1..=4)?;
                    let mut declarations = Vec::with_capacity(decl_count);
                    for _ in 0..decl_count {
                        declarations.push(CssDeclaration {
                            property: pick_dangerous_css(u)?,
                            important: false,
                        });
                    }

                    stops.push(Keyframe {
                        offset,
                        declarations,
                    });
                }

                keyframes.push(KeyframesRule {
                    name: anim.name.clone(),
                    keyframes: stops,
                });
            }
        }
    }

    Ok(keyframes)
}

fn generate_at_rules(u: &mut Unstructured) -> arbitrary::Result<Vec<AtRule>> {
    let count = u.int_in_range(0..=3)?;
    let mut rules = Vec::with_capacity(count);
    for _ in 0..count {
        rules.push(u.arbitrary()?);
    }
    Ok(rules)
}
