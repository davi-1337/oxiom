use arbitrary::Arbitrary;

/// A @font-face declaration.
#[derive(Debug, Clone, Arbitrary)]
pub struct FontFaceDecl {
    pub family: FontFamilyName,
    pub src: Vec<FontSrc>,
    pub weight: Option<FontWeightRange>,
    pub style: Option<FontStyleRange>,
    pub display: FontDisplay,
    pub unicode_range: Option<UnicodeRange>,
    pub feature_settings: Vec<OpenTypeFeature>,
    pub variation_settings: Vec<FontVariation>,
    // Phase 9: font metrics overrides
    pub size_adjust: Option<u16>,
    pub ascent_override: Option<u16>,
    pub descent_override: Option<u16>,
    pub line_gap_override: Option<u16>,
}

/// Font family name for @font-face.
#[derive(Debug, Clone)]
pub struct FontFamilyName(pub String);

impl<'a> Arbitrary<'a> for FontFamilyName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let names = [
            "FuzzFont",
            "TestSans",
            "BugSerif",
            "CrashMono",
            "ExploitDisplay",
            "GlitchUI",
            "BreakScript",
            "FaultText",
        ];
        let idx: usize = u.arbitrary()?;
        Ok(FontFamilyName(names[idx % names.len()].to_string()))
    }
}

/// Font source for @font-face src property.
#[derive(Debug, Clone, Arbitrary)]
pub enum FontSrc {
    /// local() font source.
    Local(LocalFontName),
    /// url() font source with optional format.
    Url(FontUrl),
    /// Phase 9: data URL with actual minimal valid WOFF2 bytes.
    DataUrl,
}

/// Minimal valid WOFF2 font (~60 bytes) for data URL embedding.
/// This is a minimal valid WOFF2 containing an empty glyph.
pub const MINIMAL_WOFF2_BASE64: &str = "d09GMgABAAAAAADcAAoAAAAAASAAAACPAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAABmAARABEAEQARABEAAAACwAAABIAAABdAAAAZQAAAAAAAAUGAjQBiAcHCxoGGAAdABUAPAAOAAAAAAECAwgJBAUGBw==";

#[derive(Debug, Clone)]
pub struct LocalFontName(pub String);

impl<'a> Arbitrary<'a> for LocalFontName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let names = [
            "Arial",
            "Helvetica",
            "Times New Roman",
            "Courier New",
            "Georgia",
            "Verdana",
            "Impact",
            "Comic Sans MS",
            "Trebuchet MS",
            "Lucida Console",
        ];
        let idx: usize = u.arbitrary()?;
        Ok(LocalFontName(names[idx % names.len()].to_string()))
    }
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FontUrl {
    pub format: FontFormat,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FontFormat {
    Woff2,
    Woff,
    TrueType,
    OpenType,
    Svg,
}

impl FontFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Woff2 => "woff2",
            Self::Woff => "woff",
            Self::TrueType => "truetype",
            Self::OpenType => "opentype",
            Self::Svg => "svg",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Woff2 => "woff2",
            Self::Woff => "woff",
            Self::TrueType => "ttf",
            Self::OpenType => "otf",
            Self::Svg => "svg",
        }
    }
}

/// Font weight range for @font-face.
#[derive(Debug, Clone, Arbitrary)]
pub struct FontWeightRange {
    pub start: u16,
    pub end: Option<u16>,
}

/// Font style range for @font-face.
#[derive(Debug, Clone, Arbitrary)]
pub enum FontStyleRange {
    Normal,
    Italic,
    Oblique(Option<i16>, Option<i16>),
}

/// Font-display descriptor.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FontDisplay {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

impl FontDisplay {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Block => "block",
            Self::Swap => "swap",
            Self::Fallback => "fallback",
            Self::Optional => "optional",
        }
    }
}

/// Unicode range for @font-face.
#[derive(Debug, Clone, Arbitrary)]
pub struct UnicodeRange {
    pub start: u32,
    pub end: Option<u32>,
}

/// OpenType feature tag for font-feature-settings.
#[derive(Debug, Clone, Arbitrary)]
pub struct OpenTypeFeature {
    pub tag: crate::css::OpenTypeTag,
    pub value: FeatureValue,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FeatureValue {
    On,
    Off,
    Number(u8),
}

/// Font variation axis for font-variation-settings.
#[derive(Debug, Clone, Arbitrary)]
pub struct FontVariation {
    pub tag: crate::css::VarAxisTag,
    pub value: f32,
}

/// Collection of @font-face rules and related CSS.
#[derive(Debug, Clone, Arbitrary)]
pub struct FontFaceSet {
    pub faces: Vec<FontFaceDecl>,
}
