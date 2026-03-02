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
        // Phase 5: multi-pattern — combine 2-3 templates per program
        let pattern_count = u.int_in_range(1..=3)?;
        let mut patterns = Vec::with_capacity(pattern_count);
        for _ in 0..pattern_count {
            patterns.push(u.arbitrary::<TemplatePattern>()?);
        }

        let font_faces = generate_font_faces(u)?;
        let dom = generate_dom(u)?;
        let css_rules = generate_css_rules(u, &font_faces)?;

        // Phase 1: generate keyframes for every animation in css_rules
        let keyframes = generate_keyframes(u, &css_rules)?;

        // Phase 8: generate some at-rules
        let at_rules = generate_at_rules(u)?;

        // Combine scripts from multiple patterns
        let mut script = Vec::new();
        for pattern in &patterns {
            let mut ops = generate_script(u, pattern, &font_faces)?;
            script.append(&mut ops);
        }

        // Optionally interleave operations
        if pattern_count > 1 && u.ratio(1, 3)? {
            // Shuffle operations for interleaving
            let len = script.len();
            if len > 1 {
                for i in (1..len).rev() {
                    let j = u.int_in_range(0..=i)?;
                    script.swap(i, j);
                }
            }
        }

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

/// Template patterns that focus on specific UAF trigger scenarios.
#[derive(Debug, Clone, Copy, Arbitrary)]
enum TemplatePattern {
    /// Font load + DOM removal during callback.
    FontLoadRemove,
    /// Style recalc during animation frame.
    StyleRecalcAnimation,
    /// ContentVisibility toggle + forced layout.
    ContentVisibilityToggle,
    /// Grid/flex relayout during font swap.
    GridRelayoutFontSwap,
    /// Random mix of operations.
    RandomMix,
    /// DOM tree manipulation stress.
    DomTreeStress,
    /// Font face API churn.
    FontFaceChurn,
    /// Rapid display toggling.
    DisplayToggle,
    // Phase 5: new templates
    /// 30-80 rapid-fire random ops + forced layout + GC.
    OperationStorm,
    /// Create complex state -> mutate -> layout -> destroy -> layout stale ref -> re-create.
    StateMachine,
    /// 3-6 nested rAF with different ops each frame.
    MultiFrameChain,
    /// attachShadow + innerHTML + remove host + layout.
    ShadowDomStress,
    /// MutationObserver observe, then mutate observed tree in callback.
    ObserverTrigger,
    /// Create iframe -> mutate content -> remove iframe -> layout -> GC.
    IframeCycle,
    /// contentEditable + execCommand + forced layout.
    EditingStress,
    /// Element A style change invalidates element B layout.
    CrossElementInteraction,
}

fn generate_font_faces(u: &mut Unstructured) -> arbitrary::Result<Vec<FontFaceDecl>> {
    let count = u.int_in_range(0..=4)?;
    let mut faces = Vec::with_capacity(count);
    for _ in 0..count {
        faces.push(u.arbitrary()?);
    }
    Ok(faces)
}

/// Phase 1: Generate @keyframes rules for every animation declaration in CSS rules.
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

                    // Generate declarations that mutate layout-sensitive properties
                    let decl_count = u.int_in_range(1..=4)?;
                    let mut declarations = Vec::with_capacity(decl_count);
                    for _ in 0..decl_count {
                        let prop = generate_keyframe_property(u)?;
                        declarations.push(CssDeclaration {
                            property: prop,
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

/// Generate properties that are interesting to animate for bug-finding.
fn generate_keyframe_property(u: &mut Unstructured) -> arbitrary::Result<CssProperty> {
    let choice: u8 = u.int_in_range(0..=9)?;
    Ok(match choice {
        0 => CssProperty::Display(u.arbitrary()?),
        1 => CssProperty::Width(u.arbitrary()?),
        2 => CssProperty::Height(u.arbitrary()?),
        3 => CssProperty::Transform(u.arbitrary()?),
        4 => CssProperty::ContentVisibility(u.arbitrary()?),
        5 => CssProperty::FontSize(u.arbitrary()?),
        6 => CssProperty::Opacity(u.arbitrary()?),
        7 => CssProperty::Visibility(u.arbitrary()?),
        8 => CssProperty::Position(u.arbitrary()?),
        _ => CssProperty::Overflow(u.arbitrary()?),
    })
}

/// Phase 8: Generate at-rules wrapping CSS rules.
fn generate_at_rules(u: &mut Unstructured) -> arbitrary::Result<Vec<AtRule>> {
    let count = u.int_in_range(0..=2)?;
    let mut rules = Vec::with_capacity(count);
    for _ in 0..count {
        rules.push(u.arbitrary()?);
    }
    Ok(rules)
}

/// Phase 2: generate larger DOM trees with configurable shape.
fn generate_dom(u: &mut Unstructured) -> arbitrary::Result<DomTree> {
    // 10% wide tree mode, 10% deep tree mode, 80% normal
    let mode: u8 = u.int_in_range(0..=9)?;

    let (root_count, max_depth, max_children) = match mode {
        0 => {
            // Wide tree: 30-50 root children, depth 2
            let count = u.int_in_range(30..=50)?;
            (count, 2usize, 2usize)
        }
        1 => {
            // Deep tree: depth 7, 1-2 children/level
            let count = u.int_in_range(1..=3)?;
            (count, 7usize, 2usize)
        }
        _ => {
            // Normal: 3-20 root children, depth 5, 0-5 children
            let count = u.int_in_range(3..=20)?;
            (count, 5usize, 5usize)
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
    let count = u.int_in_range(1..=12)?;
    let mut rules = Vec::with_capacity(count);

    for _ in 0..count {
        let selector: Selector = u.arbitrary()?;
        let decl_count = u.int_in_range(1..=6)?;
        let mut declarations = Vec::with_capacity(decl_count);

        for _ in 0..decl_count {
            // Sometimes use boundary values
            let property = if u.ratio(1, 3)? {
                generate_boundary_property(u)?
            } else {
                u.arbitrary()?
            };

            // Cross-reference font families from font_faces
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
            let values = boundary::boundary_font_sizes();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::FontSize(values[idx].clone())
        }
        1 => {
            let values = boundary::boundary_displays();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::Display(values[idx])
        }
        2 => {
            let values = boundary::boundary_positions();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::Position(values[idx])
        }
        3 => {
            let values = boundary::boundary_content_visibilities();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::ContentVisibility(values[idx])
        }
        4 => {
            let values = boundary::boundary_contains();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::Contain(values[idx])
        }
        5 => {
            let values = boundary::boundary_font_weights();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::FontWeight(values[idx].clone())
        }
        6 => {
            let values = boundary::boundary_z_indices();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::ZIndex(values[idx].clone())
        }
        7 => {
            let values = boundary::boundary_dimensions();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::Width(values[idx].clone())
        }
        8 => {
            let values = boundary::boundary_opacities();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::Opacity(values[idx].clone())
        }
        9 => {
            let values = boundary::boundary_will_changes();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::WillChange(values[idx])
        }
        // Phase 6: new boundary properties
        10 => {
            let values = boundary::boundary_border_widths();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::BorderWidth(values[idx].clone())
        }
        11 => {
            let values = boundary::boundary_border_radii();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::BorderRadius(values[idx].clone())
        }
        12 => {
            let values = boundary::boundary_colors();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::Color(values[idx].clone())
        }
        13 => {
            let values = boundary::boundary_colors();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::BackgroundColor(values[idx].clone())
        }
        14 => {
            let values = boundary::boundary_border_styles();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            CssProperty::BorderStyle(values[idx])
        }
        _ => {
            let values = boundary::boundary_global_keywords();
            let idx = u.int_in_range(0..=values.len() - 1)?;
            let prop: CssPropertyName = u.arbitrary()?;
            CssProperty::GlobalReset(prop, values[idx])
        }
    })
}

fn generate_script(
    u: &mut Unstructured,
    pattern: &TemplatePattern,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<JsOperation>> {
    match pattern {
        TemplatePattern::FontLoadRemove => generate_font_load_remove(u, font_faces),
        TemplatePattern::StyleRecalcAnimation => generate_style_recalc_animation(u),
        TemplatePattern::ContentVisibilityToggle => generate_content_visibility_toggle(u),
        TemplatePattern::GridRelayoutFontSwap => generate_grid_relayout_font_swap(u, font_faces),
        TemplatePattern::RandomMix => generate_random_mix(u),
        TemplatePattern::DomTreeStress => generate_dom_tree_stress(u),
        TemplatePattern::FontFaceChurn => generate_font_face_churn(u, font_faces),
        TemplatePattern::DisplayToggle => generate_display_toggle(u),
        // Phase 5: new templates
        TemplatePattern::OperationStorm => generate_operation_storm(u),
        TemplatePattern::StateMachine => generate_state_machine(u),
        TemplatePattern::MultiFrameChain => generate_multi_frame_chain(u),
        TemplatePattern::ShadowDomStress => generate_shadow_dom_stress(u),
        TemplatePattern::ObserverTrigger => generate_observer_trigger(u),
        TemplatePattern::IframeCycle => generate_iframe_cycle(u),
        TemplatePattern::EditingStress => generate_editing_stress(u),
        TemplatePattern::CrossElementInteraction => generate_cross_element_interaction(u),
    }
}

/// Font load + DOM removal during callback.
fn generate_font_load_remove(
    u: &mut Unstructured,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    for face in font_faces {
        ops.push(JsOperation::FontFaceLoad {
            family: String8(face.family.0.clone()),
        });
    }

    let target: NodeRef = u.arbitrary()?;
    ops.push(JsOperation::GetOffsetWidth { target });

    let remove_target: NodeRef = u.arbitrary()?;
    let layout_target: NodeRef = u.arbitrary()?;
    ops.push(JsOperation::RequestAnimationFrame {
        operations: vec![
            JsOperation::RemoveChild {
                target: remove_target,
            },
            JsOperation::ForceGC,
            JsOperation::GetOffsetWidth {
                target: layout_target,
            },
        ],
    });

    let extra_count = u.int_in_range(0..=3)?;
    for _ in 0..extra_count {
        ops.push(u.arbitrary()?);
    }

    Ok(ops)
}

/// Style recalc during animation frame.
fn generate_style_recalc_animation(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let target: NodeRef = u.arbitrary()?;
    let property: CssProperty = u.arbitrary()?;

    ops.push(JsOperation::SetInlineStyle {
        target,
        property: property.clone(),
    });

    let new_property: CssProperty = u.arbitrary()?;
    let layout_target: NodeRef = u.arbitrary()?;
    ops.push(JsOperation::RequestAnimationFrame {
        operations: vec![
            JsOperation::SetInlineStyle {
                target,
                property: new_property,
            },
            JsOperation::GetOffsetWidth {
                target: layout_target,
            },
            JsOperation::ForceGC,
        ],
    });

    let another_prop: CssProperty = u.arbitrary()?;
    ops.push(JsOperation::RequestAnimationFrame {
        operations: vec![JsOperation::RequestAnimationFrame {
            operations: vec![
                JsOperation::SetInlineStyle {
                    target,
                    property: another_prop,
                },
                JsOperation::GetBoundingClientRect { target },
            ],
        }],
    });

    Ok(ops)
}

/// ContentVisibility toggle + forced layout.
fn generate_content_visibility_toggle(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();
    let target: NodeRef = u.arbitrary()?;

    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::ContentVisibility(ContentVisibilityValue::Auto),
    });
    ops.push(JsOperation::GetOffsetWidth { target });
    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::ContentVisibility(ContentVisibilityValue::Hidden),
    });
    ops.push(JsOperation::GetOffsetHeight { target });
    ops.push(JsOperation::SetTimeout {
        delay_ms: 0,
        operations: vec![
            JsOperation::SetInlineStyle {
                target,
                property: CssProperty::ContentVisibility(ContentVisibilityValue::Visible),
            },
            JsOperation::GetBoundingClientRect { target },
            JsOperation::ForceGC,
        ],
    });
    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::Contain(ContainValue::Strict),
    });
    ops.push(JsOperation::GetOffsetWidth { target });

    let extra: usize = u.int_in_range(0..=2)?;
    for _ in 0..extra {
        ops.push(u.arbitrary()?);
    }

    Ok(ops)
}

/// Grid/flex relayout during font swap.
fn generate_grid_relayout_font_swap(
    u: &mut Unstructured,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();
    let target: NodeRef = u.arbitrary()?;

    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::Display(DisplayValue::Grid),
    });

    let template: GridTemplateValue = u.arbitrary()?;
    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::GridTemplateColumns(template),
    });

    if let Some(face) = font_faces.first() {
        ops.push(JsOperation::SetInlineStyle {
            target,
            property: CssProperty::FontFamily(FontFamilyValue::Named(String8(
                face.family.0.clone(),
            ))),
        });
    }

    ops.push(JsOperation::GetOffsetWidth { target });

    let child: NodeRef = u.arbitrary()?;
    ops.push(JsOperation::RequestAnimationFrame {
        operations: vec![
            JsOperation::SetInlineStyle {
                target,
                property: CssProperty::FontFamily(FontFamilyValue::SansSerif),
            },
            JsOperation::GetOffsetWidth { target: child },
            JsOperation::SetInlineStyle {
                target,
                property: CssProperty::Display(DisplayValue::Flex),
            },
            JsOperation::GetBoundingClientRect { target },
            JsOperation::ForceGC,
        ],
    });

    Ok(ops)
}

/// Random mix of operations.
fn generate_random_mix(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let count = u.int_in_range(2..=12)?;
    let mut ops = Vec::with_capacity(count);
    for _ in 0..count {
        ops.push(u.arbitrary()?);
    }
    Ok(ops)
}

/// DOM tree manipulation stress.
fn generate_dom_tree_stress(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let count = u.int_in_range(3..=8)?;
    for _ in 0..count {
        let choice: u8 = u.int_in_range(0..=6)?;
        let op = match choice {
            0 => JsOperation::AppendChild {
                parent: u.arbitrary()?,
                child: u.arbitrary()?,
            },
            1 => JsOperation::RemoveChild {
                target: u.arbitrary()?,
            },
            2 => JsOperation::CloneNode {
                source: u.arbitrary()?,
                deep: u.arbitrary()?,
                append_to: u.arbitrary()?,
            },
            3 => JsOperation::SetInnerHTML {
                target: u.arbitrary()?,
                html: u.arbitrary()?,
            },
            4 => JsOperation::RangeDeleteContents {
                start_node: u.arbitrary()?,
                end_node: u.arbitrary()?,
            },
            5 => JsOperation::InsertBefore {
                parent: u.arbitrary()?,
                new_child: u.arbitrary()?,
                ref_child: u.arbitrary()?,
            },
            _ => JsOperation::ReplaceChild {
                parent: u.arbitrary()?,
                new_child: u.arbitrary()?,
                old_child: u.arbitrary()?,
            },
        };
        ops.push(op);

        if u.ratio(1, 2)? {
            ops.push(JsOperation::GetOffsetWidth {
                target: u.arbitrary()?,
            });
        }
    }

    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::GetOffsetWidth {
        target: u.arbitrary()?,
    });

    Ok(ops)
}

/// Font face API churn.
fn generate_font_face_churn(
    u: &mut Unstructured,
    font_faces: &[FontFaceDecl],
) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let count = u.int_in_range(2..=6)?;
    for _ in 0..count {
        let family: String8 = u.arbitrary()?;
        let source: FontFaceSource = u.arbitrary()?;

        ops.push(JsOperation::FontFaceAdd {
            family: family.clone(),
            source,
        });
        ops.push(JsOperation::FontFaceLoad {
            family: family.clone(),
        });
        ops.push(JsOperation::GetOffsetWidth {
            target: u.arbitrary()?,
        });
        ops.push(JsOperation::FontFaceRemove { family });
        ops.push(JsOperation::ForceGC);
    }

    for face in font_faces {
        ops.push(JsOperation::FontFaceLoad {
            family: String8(face.family.0.clone()),
        });
    }

    Ok(ops)
}

/// Rapid display toggling.
fn generate_display_toggle(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();
    let target: NodeRef = u.arbitrary()?;
    let displays = boundary::boundary_displays();

    for display in &displays {
        ops.push(JsOperation::SetInlineStyle {
            target,
            property: CssProperty::Display(*display),
        });
        ops.push(JsOperation::GetOffsetWidth { target });
    }

    let raf_target: NodeRef = u.arbitrary()?;
    ops.push(JsOperation::RequestAnimationFrame {
        operations: vec![
            JsOperation::SetInlineStyle {
                target: raf_target,
                property: CssProperty::Display(DisplayValue::None),
            },
            JsOperation::ForceGC,
            JsOperation::SetInlineStyle {
                target: raf_target,
                property: CssProperty::Display(DisplayValue::Contents),
            },
            JsOperation::GetBoundingClientRect {
                target: raf_target,
            },
        ],
    });

    Ok(ops)
}

// ============================
// Phase 5: New template patterns
// ============================

/// OperationStorm: 30-80 rapid-fire random ops + forced layout + GC.
fn generate_operation_storm(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let count = u.int_in_range(30..=80)?;
    let mut ops = Vec::with_capacity(count + 2);

    for _ in 0..count {
        ops.push(u.arbitrary()?);
    }

    // Force layout after storm
    ops.push(JsOperation::GetOffsetWidth {
        target: u.arbitrary()?,
    });
    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::GetBoundingClientRect {
        target: u.arbitrary()?,
    });

    Ok(ops)
}

/// StateMachine: create complex state -> mutate -> layout -> destroy -> layout stale ref -> re-create via innerHTML.
fn generate_state_machine(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let target: NodeRef = u.arbitrary()?;
    let child: NodeRef = u.arbitrary()?;

    // Create complex state
    ops.push(JsOperation::SetInlineStyle {
        target,
        property: CssProperty::Display(DisplayValue::Grid),
    });
    ops.push(JsOperation::AppendChild {
        parent: target,
        child,
    });
    ops.push(JsOperation::SetInlineStyle {
        target: child,
        property: CssProperty::ContentVisibility(ContentVisibilityValue::Auto),
    });

    // Mutate
    ops.push(JsOperation::SetInnerHTML {
        target: child,
        html: u.arbitrary()?,
    });
    ops.push(JsOperation::GetOffsetWidth { target });

    // Destroy
    ops.push(JsOperation::RemoveChild { target: child });
    ops.push(JsOperation::ForceGC);

    // Layout stale reference
    ops.push(JsOperation::GetOffsetWidth { target: child });
    ops.push(JsOperation::GetBoundingClientRect { target });

    // Re-create via innerHTML
    ops.push(JsOperation::SetInnerHTML {
        target,
        html: u.arbitrary()?,
    });
    ops.push(JsOperation::GetOffsetWidth { target });
    ops.push(JsOperation::ForceGC);

    Ok(ops)
}

/// MultiFrameChain: 3-6 nested rAF with different ops each frame.
fn generate_multi_frame_chain(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let depth = u.int_in_range(3..=6)?;
    let target: NodeRef = u.arbitrary()?;

    // Build from inside out
    let mut inner_ops: Vec<JsOperation> = vec![
        JsOperation::ForceGC,
        JsOperation::GetOffsetWidth { target },
    ];

    for _ in 0..depth {
        let prop: CssProperty = u.arbitrary()?;
        let mut frame_ops = vec![
            JsOperation::SetInlineStyle {
                target,
                property: prop,
            },
            JsOperation::GetOffsetWidth { target },
        ];
        frame_ops.push(JsOperation::RequestAnimationFrame {
            operations: inner_ops,
        });
        inner_ops = frame_ops;
    }

    Ok(vec![JsOperation::RequestAnimationFrame {
        operations: inner_ops,
    }])
}

/// ShadowDomStress: attachShadow + innerHTML + remove host + layout.
fn generate_shadow_dom_stress(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let host: NodeRef = u.arbitrary()?;
    let mode: ShadowRootMode = u.arbitrary()?;

    // Attach shadow root
    ops.push(JsOperation::AttachShadowRoot {
        target: host,
        mode,
    });

    // Set shadow root innerHTML
    ops.push(JsOperation::ShadowRootSetInnerHTML {
        host,
        html: u.arbitrary()?,
    });

    // Force layout
    ops.push(JsOperation::GetOffsetWidth { target: host });

    // Mutate shadow content
    ops.push(JsOperation::ShadowRootSetInnerHTML {
        host,
        html: u.arbitrary()?,
    });
    ops.push(JsOperation::GetBoundingClientRect { target: host });

    // Remove host
    ops.push(JsOperation::RemoveChild { target: host });
    ops.push(JsOperation::ForceGC);

    // Layout stale reference
    ops.push(JsOperation::GetOffsetWidth { target: host });

    let extra: usize = u.int_in_range(0..=3)?;
    for _ in 0..extra {
        ops.push(u.arbitrary()?);
    }

    Ok(ops)
}

/// ObserverTrigger: MutationObserver observe, then mutate observed tree in callback.
fn generate_observer_trigger(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let observe_target: NodeRef = u.arbitrary()?;
    let mutate_target: NodeRef = u.arbitrary()?;

    // Observe and mutate
    ops.push(JsOperation::ObserveAndMutate {
        observe_target,
        mutate_target,
        mutation_op: u.arbitrary()?,
    });

    // Force layout
    ops.push(JsOperation::GetOffsetWidth {
        target: observe_target,
    });

    // Additional mutations
    ops.push(JsOperation::SetInnerHTML {
        target: mutate_target,
        html: u.arbitrary()?,
    });
    ops.push(JsOperation::GetBoundingClientRect {
        target: observe_target,
    });

    // ResizeObserver
    let resize_target: NodeRef = u.arbitrary()?;
    ops.push(JsOperation::ResizeObserverObserve {
        target: resize_target,
        callback_ops: vec![
            JsOperation::SetInlineStyle {
                target: resize_target,
                property: CssProperty::Width(LengthOrAuto::Length(LengthValue::Px(200))),
            },
            JsOperation::GetOffsetWidth {
                target: resize_target,
            },
        ],
    });

    // Trigger resize
    ops.push(JsOperation::SetInlineStyle {
        target: resize_target,
        property: CssProperty::Width(LengthOrAuto::Length(LengthValue::Px(100))),
    });

    ops.push(JsOperation::ForceGC);

    Ok(ops)
}

/// IframeCycle: create iframe -> mutate content -> remove iframe -> layout -> GC.
fn generate_iframe_cycle(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let count = u.int_in_range(2..=4)?;
    for _ in 0..count {
        let target: NodeRef = u.arbitrary()?;

        // Create iframe
        ops.push(JsOperation::CreateIframe {
            target,
            src_html: u.arbitrary()?,
        });
        ops.push(JsOperation::GetOffsetWidth { target });

        // Remove iframe
        ops.push(JsOperation::RemoveIframe { target });
        ops.push(JsOperation::ForceGC);
        ops.push(JsOperation::GetOffsetWidth { target });
    }

    Ok(ops)
}

/// EditingStress: contentEditable + execCommand + forced layout.
fn generate_editing_stress(u: &mut Unstructured) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let target: NodeRef = u.arbitrary()?;

    // Make element contentEditable
    ops.push(JsOperation::SetAttribute {
        target,
        attr_name: AttrName::ContentEditable,
        attr_value: String8("true".to_string()),
    });

    // Focus it
    ops.push(JsOperation::DispatchEvent {
        target,
        event_type: EventType::Focus,
    });

    // Select all content
    ops.push(JsOperation::SelectAllContent);

    // Execute various commands
    let cmd_count = u.int_in_range(3..=8)?;
    for _ in 0..cmd_count {
        let cmd: ExecCommandType = u.arbitrary()?;
        ops.push(JsOperation::ExecCommand { command: cmd });
        ops.push(JsOperation::GetOffsetWidth { target });
    }

    ops.push(JsOperation::ForceGC);
    ops.push(JsOperation::GetBoundingClientRect { target });

    Ok(ops)
}

/// CrossElementInteraction: element A style change invalidates element B layout.
fn generate_cross_element_interaction(
    u: &mut Unstructured,
) -> arbitrary::Result<Vec<JsOperation>> {
    let mut ops = Vec::new();

    let elem_a: NodeRef = u.arbitrary()?;
    let elem_b: NodeRef = u.arbitrary()?;
    let elem_c: NodeRef = u.arbitrary()?;

    // Set up parent-child relationship
    ops.push(JsOperation::AppendChild {
        parent: elem_a,
        child: elem_b,
    });

    // Style A affects B's layout
    ops.push(JsOperation::SetInlineStyle {
        target: elem_a,
        property: CssProperty::Display(DisplayValue::Flex),
    });
    ops.push(JsOperation::GetOffsetWidth { target: elem_b });

    // Change A's style, read B's layout
    ops.push(JsOperation::SetInlineStyle {
        target: elem_a,
        property: CssProperty::Display(DisplayValue::Grid),
    });
    ops.push(JsOperation::GetBoundingClientRect { target: elem_b });

    // Reparent B under C
    ops.push(JsOperation::AppendChild {
        parent: elem_c,
        child: elem_b,
    });
    ops.push(JsOperation::GetOffsetWidth { target: elem_b });

    // Style change on C
    ops.push(JsOperation::SetInlineStyle {
        target: elem_c,
        property: CssProperty::ContentVisibility(ContentVisibilityValue::Hidden),
    });
    ops.push(JsOperation::GetOffsetWidth { target: elem_b });

    ops.push(JsOperation::ForceGC);

    // QueueMicrotask for timing-sensitive mutations
    ops.push(JsOperation::QueueMicrotask {
        operations: vec![
            JsOperation::SetInlineStyle {
                target: elem_a,
                property: CssProperty::Display(DisplayValue::None),
            },
            JsOperation::GetOffsetWidth { target: elem_b },
        ],
    });

    Ok(ops)
}
