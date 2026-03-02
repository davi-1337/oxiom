use arbitrary::Arbitrary;

use crate::css::{CssProperty, String8};
use crate::html::NodeRef;

/// JavaScript operations for DOM/CSS mutation during fuzzing.
#[derive(Debug, Clone, Arbitrary)]
pub enum JsOperation {
    /// Set an inline style on a node.
    SetInlineStyle {
        target: NodeRef,
        property: CssProperty,
    },

    /// Remove an inline style from a node.
    RemoveInlineStyle {
        target: NodeRef,
        property_name: StylePropertyName,
    },

    /// Insert a CSS rule into a stylesheet.
    InsertStyleRule {
        rule_text: InsertRuleText,
        index: u8,
    },

    /// Delete a CSS rule from a stylesheet.
    DeleteStyleRule {
        index: u8,
    },

    /// Trigger FontFace.load().
    FontFaceLoad {
        family: String8,
    },

    /// Add a FontFace via JS FontFace API.
    FontFaceAdd {
        family: String8,
        source: FontFaceSource,
    },

    /// Remove all fonts matching family from document.fonts.
    FontFaceRemove {
        family: String8,
    },

    /// Append a child node to a parent.
    AppendChild {
        parent: NodeRef,
        child: NodeRef,
    },

    /// Remove a child from its parent.
    RemoveChild {
        target: NodeRef,
    },

    /// Clone a node (optionally deep).
    CloneNode {
        source: NodeRef,
        deep: bool,
        append_to: NodeRef,
    },

    /// Read offsetWidth to force layout.
    GetOffsetWidth {
        target: NodeRef,
    },

    /// Read offsetHeight to force layout.
    GetOffsetHeight {
        target: NodeRef,
    },

    /// Read getBoundingClientRect to force layout.
    GetBoundingClientRect {
        target: NodeRef,
    },

    /// Read getComputedStyle to force style recalc.
    GetComputedStyle {
        target: NodeRef,
        property_name: StylePropertyName,
    },

    /// Force garbage collection via gc().
    ForceGC,

    /// Wrap next operations in requestAnimationFrame.
    RequestAnimationFrame {
        operations: Vec<JsOperation>,
    },

    /// Wrap next operations in setTimeout.
    SetTimeout {
        delay_ms: u16,
        operations: Vec<JsOperation>,
    },

    /// Set innerHTML on a node.
    SetInnerHTML {
        target: NodeRef,
        html: InnerHtmlContent,
    },

    /// Set textContent on a node.
    SetTextContent {
        target: NodeRef,
        text: String8,
    },

    /// Toggle a class on a node.
    ToggleClass {
        target: NodeRef,
        class_name: String8,
    },

    /// Set an attribute on a node.
    SetAttribute {
        target: NodeRef,
        attr_name: AttrName,
        attr_value: String8,
    },

    /// Remove an attribute from a node.
    RemoveAttribute {
        target: NodeRef,
        attr_name: AttrName,
    },

    /// Create and dispatch a custom event.
    DispatchEvent {
        target: NodeRef,
        event_type: EventType,
    },

    /// document.adoptNode on an element.
    AdoptNode {
        target: NodeRef,
    },

    /// Insert adjacent HTML.
    InsertAdjacentHTML {
        target: NodeRef,
        position: InsertPosition,
        html: InnerHtmlContent,
    },

    /// Range-based DOM manipulation.
    RangeDeleteContents {
        start_node: NodeRef,
        end_node: NodeRef,
    },

    // ========================================
    // Phase 3: Critical JS operations
    // ========================================

    /// Attach a shadow root to an element.
    AttachShadowRoot {
        target: NodeRef,
        mode: ShadowRootMode,
    },

    /// Set innerHTML on a shadow root.
    ShadowRootSetInnerHTML {
        host: NodeRef,
        html: InnerHtmlContent,
    },

    /// Create a MutationObserver, observe a target, then mutate.
    ObserveAndMutate {
        observe_target: NodeRef,
        mutate_target: NodeRef,
        mutation_op: MutationOp,
    },

    /// Insert a node before a reference child.
    InsertBefore {
        parent: NodeRef,
        new_child: NodeRef,
        ref_child: NodeRef,
    },

    /// Replace a child node with another.
    ReplaceChild {
        parent: NodeRef,
        new_child: NodeRef,
        old_child: NodeRef,
    },

    /// Replace this element with HTML content.
    ReplaceWith {
        target: NodeRef,
        html: InnerHtmlContent,
    },

    /// Execute a document editing command.
    ExecCommand {
        command: ExecCommandType,
    },

    /// Select all document content.
    SelectAllContent,

    /// Collapse selection to a target node.
    CollapseSelection {
        target: NodeRef,
    },

    /// Create an iframe and insert it into the DOM.
    CreateIframe {
        target: NodeRef,
        src_html: InnerHtmlContent,
    },

    /// Remove an iframe from the DOM.
    RemoveIframe {
        target: NodeRef,
    },

    /// Create a ResizeObserver and observe a target.
    ResizeObserverObserve {
        target: NodeRef,
        callback_ops: Vec<JsOperation>,
    },

    /// Create an IntersectionObserver and observe a target.
    IntersectionObserverObserve {
        target: NodeRef,
        callback_ops: Vec<JsOperation>,
    },

    /// Queue a microtask with operations.
    QueueMicrotask {
        operations: Vec<JsOperation>,
    },
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ShadowRootMode {
    Open,
    Closed,
}

/// Simplified mutation operation for MutationObserver callbacks.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum MutationOp {
    SetAttribute,
    RemoveChild,
    AppendChild,
    SetInnerHTML,
    SetTextContent,
}

/// Document editing commands for execCommand.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ExecCommandType {
    InsertHTML,
    Delete,
    Bold,
    Italic,
    InsertText,
    InsertUnorderedList,
    InsertOrderedList,
    Indent,
    Outdent,
    FormatBlock,
    RemoveFormat,
    SelectAll,
}

impl ExecCommandType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InsertHTML => "insertHTML",
            Self::Delete => "delete",
            Self::Bold => "bold",
            Self::Italic => "italic",
            Self::InsertText => "insertText",
            Self::InsertUnorderedList => "insertUnorderedList",
            Self::InsertOrderedList => "insertOrderedList",
            Self::Indent => "indent",
            Self::Outdent => "outdent",
            Self::FormatBlock => "formatBlock",
            Self::RemoveFormat => "removeFormat",
            Self::SelectAll => "selectAll",
        }
    }

    pub fn needs_value(&self) -> bool {
        matches!(self, Self::InsertHTML | Self::InsertText | Self::FormatBlock)
    }

    pub fn default_value(&self) -> &'static str {
        match self {
            Self::InsertHTML => "<b>fuzz</b>",
            Self::InsertText => "fuzz",
            Self::FormatBlock => "<h1>",
            _ => "",
        }
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum StylePropertyName {
    Display,
    FontFamily,
    FontSize,
    FontWeight,
    Position,
    Width,
    Height,
    Transform,
    Opacity,
    Visibility,
    Overflow,
    ContentVisibility,
    Contain,
    WillChange,
}

impl StylePropertyName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::FontFamily => "font-family",
            Self::FontSize => "font-size",
            Self::FontWeight => "font-weight",
            Self::Position => "position",
            Self::Width => "width",
            Self::Height => "height",
            Self::Transform => "transform",
            Self::Opacity => "opacity",
            Self::Visibility => "visibility",
            Self::Overflow => "overflow",
            Self::ContentVisibility => "content-visibility",
            Self::Contain => "contain",
            Self::WillChange => "will-change",
        }
    }

    pub fn as_camel_case(&self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::FontFamily => "fontFamily",
            Self::FontSize => "fontSize",
            Self::FontWeight => "fontWeight",
            Self::Position => "position",
            Self::Width => "width",
            Self::Height => "height",
            Self::Transform => "transform",
            Self::Opacity => "opacity",
            Self::Visibility => "visibility",
            Self::Overflow => "overflow",
            Self::ContentVisibility => "contentVisibility",
            Self::Contain => "contain",
            Self::WillChange => "willChange",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertRuleText(pub String);

impl<'a> Arbitrary<'a> for InsertRuleText {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let rules = [
            "div { display: none }",
            "span { font-size: 0 }",
            ".a { content-visibility: auto }",
            "* { contain: strict }",
            "td { display: contents }",
            "p { font-family: FuzzFont }",
        ];
        let idx: usize = u.arbitrary()?;
        Ok(InsertRuleText(rules[idx % rules.len()].to_string()))
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FontFaceSource {
    LocalArial,
    LocalTimesNewRoman,
    EmptyUrl,
    DataUrl,
}

#[derive(Debug, Clone)]
pub struct InnerHtmlContent(pub String);

impl<'a> Arbitrary<'a> for InnerHtmlContent {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let contents = [
            "<div>x</div>",
            "<span style='display:none'>y</span>",
            "<table><tr><td>z</td></tr></table>",
            "",
            "<svg><rect/></svg>",
            "<details><summary>s</summary></details>",
        ];
        let idx: usize = u.arbitrary()?;
        Ok(InnerHtmlContent(contents[idx % contents.len()].to_string()))
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum AttrName {
    Style,
    Class,
    Id,
    Hidden,
    ContentEditable,
    Dir,
    Lang,
    Tabindex,
}

impl AttrName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Style => "style",
            Self::Class => "class",
            Self::Id => "id",
            Self::Hidden => "hidden",
            Self::ContentEditable => "contenteditable",
            Self::Dir => "dir",
            Self::Lang => "lang",
            Self::Tabindex => "tabindex",
        }
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum EventType {
    Click,
    Focus,
    Blur,
    Resize,
    Scroll,
    Transitionend,
    Animationend,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::Focus => "focus",
            Self::Blur => "blur",
            Self::Resize => "resize",
            Self::Scroll => "scroll",
            Self::Transitionend => "transitionend",
            Self::Animationend => "animationend",
        }
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum InsertPosition {
    BeforeBegin,
    AfterBegin,
    BeforeEnd,
    AfterEnd,
}

impl InsertPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BeforeBegin => "beforebegin",
            Self::AfterBegin => "afterbegin",
            Self::BeforeEnd => "beforeend",
            Self::AfterEnd => "afterend",
        }
    }
}
