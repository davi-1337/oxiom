use arbitrary::Arbitrary;

/// Reference to a node by index in the flat node list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Arbitrary)]
pub struct NodeRef(pub u16);

/// HTML element types relevant to CSS/font fuzzing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Arbitrary)]
pub enum HtmlElement {
    Div,
    Span,
    P,
    A,
    H1,
    H2,
    H3,
    Section,
    Article,
    Main,
    Nav,
    Aside,
    Header,
    Footer,
    Table,
    Tr,
    Td,
    Th,
    Ul,
    Ol,
    Li,
    Dl,
    Dt,
    Dd,
    Pre,
    Code,
    Blockquote,
    Figure,
    Figcaption,
    Details,
    Summary,
    Canvas,
    Svg,
    Input,
    Textarea,
    Select,
    Button,
    Label,
    Fieldset,
    Legend,
    // Phase 2: new elements
    Iframe,
    Dialog,
    Slot,
    Template,
    Ruby,
    Rt,
    Video,
    Img,
}

impl HtmlElement {
    pub fn tag_name(&self) -> &'static str {
        match self {
            Self::Div => "div",
            Self::Span => "span",
            Self::P => "p",
            Self::A => "a",
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::Section => "section",
            Self::Article => "article",
            Self::Main => "main",
            Self::Nav => "nav",
            Self::Aside => "aside",
            Self::Header => "header",
            Self::Footer => "footer",
            Self::Table => "table",
            Self::Tr => "tr",
            Self::Td => "td",
            Self::Th => "th",
            Self::Ul => "ul",
            Self::Ol => "ol",
            Self::Li => "li",
            Self::Dl => "dl",
            Self::Dt => "dt",
            Self::Dd => "dd",
            Self::Pre => "pre",
            Self::Code => "code",
            Self::Blockquote => "blockquote",
            Self::Figure => "figure",
            Self::Figcaption => "figcaption",
            Self::Details => "details",
            Self::Summary => "summary",
            Self::Canvas => "canvas",
            Self::Svg => "svg",
            Self::Input => "input",
            Self::Textarea => "textarea",
            Self::Select => "select",
            Self::Button => "button",
            Self::Label => "label",
            Self::Fieldset => "fieldset",
            Self::Legend => "legend",
            Self::Iframe => "iframe",
            Self::Dialog => "dialog",
            Self::Slot => "slot",
            Self::Template => "template",
            Self::Ruby => "ruby",
            Self::Rt => "rt",
            Self::Video => "video",
            Self::Img => "img",
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Input | Self::Img)
    }
}

/// A node in the DOM tree.
#[derive(Debug, Clone, Arbitrary)]
pub struct DomNode {
    pub element: HtmlElement,
    pub text_content: Option<TextContent>,
    pub children: Vec<DomNode>,
}

/// Text content for a node (limited length for sanity).
#[derive(Debug, Clone)]
pub struct TextContent(pub String);

impl<'a> Arbitrary<'a> for TextContent {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let options = [
            "test", "fuzz", "AAAA", "\u{00AD}", "\u{200B}", "\u{FEFF}",
            "\u{0041}\u{0301}", "\u{1F600}", "a\u{0308}b", "",
            "\u{202E}abc", "\u{FFFD}", "x\u{0000}y",
            // Phase 10: richer text content
            "\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}", // Arabic
            "\u{4E2D}\u{6587}\u{6D4B}\u{8BD5}",                         // CJK
            "\u{0E17}\u{0E14}\u{0E2A}\u{0E2D}\u{0E1A}",                 // Thai
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}", // complex emoji
            "ffi\u{FB01}\u{FB02}",                                       // ligature-heavy Latin
            "\u{0639}\u{0631}\u{0628}\u{064A}text\u{05E2}\u{05D1}\u{05E8}\u{05D9}\u{05EA}", // bidi mix
            "\u{1F1FA}\u{1F1F8}\u{1F3F3}\u{FE0F}\u{200D}\u{1F308}",     // flag + rainbow emoji
        ];
        let idx: usize = u.arbitrary()?;
        Ok(TextContent(options[idx % options.len()].to_string()))
    }
}

/// The full DOM tree for a fuzz program.
#[derive(Debug, Clone, Arbitrary)]
pub struct DomTree {
    pub root_children: Vec<DomNode>,
}
