use arbitrary::Arbitrary;

use crate::html::NodeRef;

/// CSS selector.
#[derive(Debug, Clone, Arbitrary)]
pub enum Selector {
    /// Select by tag name.
    Tag(TagSelector),
    /// Select by class.
    Class(String8),
    /// Select by id (references a node).
    Id(NodeRef),
    /// Pseudo-class selector.
    Pseudo(PseudoSelector),
    /// Combinator of two selectors.
    Combinator(Box<CombinatorSelector>),
    /// Universal selector.
    Universal,
    // Phase 8: advanced selectors
    /// :has() pseudo-class.
    Has(Box<Selector>),
    /// :not() pseudo-class.
    Not(Box<Selector>),
    /// :is() pseudo-class.
    Is(Box<Selector>),
    /// :where() pseudo-class.
    Where(Box<Selector>),
    /// Attribute selector.
    Attribute(AttrSelector),
}

#[derive(Debug, Clone, Arbitrary)]
pub struct CombinatorSelector {
    pub left: Selector,
    pub combinator: CombinatorKind,
    pub right: Selector,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum CombinatorKind {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

impl CombinatorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Descendant => " ",
            Self::Child => " > ",
            Self::AdjacentSibling => " + ",
            Self::GeneralSibling => " ~ ",
        }
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TagSelector {
    Div,
    Span,
    P,
    Table,
    Td,
    Svg,
    Canvas,
    Details,
    Summary,
    Pre,
}

impl TagSelector {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Div => "div",
            Self::Span => "span",
            Self::P => "p",
            Self::Table => "table",
            Self::Td => "td",
            Self::Svg => "svg",
            Self::Canvas => "canvas",
            Self::Details => "details",
            Self::Summary => "summary",
            Self::Pre => "pre",
        }
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum PseudoSelector {
    Hover,
    Focus,
    Active,
    FirstChild,
    LastChild,
    NthChild(u8),
    Before,
    After,
    Empty,
    Root,
    // Phase 8
    FocusWithin,
}

/// Attribute selector for Phase 8.
#[derive(Debug, Clone, Arbitrary)]
pub struct AttrSelector {
    pub attr: AttrSelectorName,
    pub op: Option<AttrSelectorOp>,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum AttrSelectorName {
    Id,
    Class,
    Style,
    Hidden,
    ContentEditable,
    Dir,
    Lang,
}

impl AttrSelectorName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Class => "class",
            Self::Style => "style",
            Self::Hidden => "hidden",
            Self::ContentEditable => "contenteditable",
            Self::Dir => "dir",
            Self::Lang => "lang",
        }
    }
}

#[derive(Debug, Clone, Arbitrary)]
pub struct AttrSelectorOp {
    pub kind: AttrSelectorOpKind,
    pub value: String8,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum AttrSelectorOpKind {
    Equals,
    Contains,
    StartsWith,
}

/// Short string (max 8 chars) for class names etc.
#[derive(Debug, Clone)]
pub struct String8(pub String);

impl<'a> Arbitrary<'a> for String8 {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let names = ["a", "b", "c", "x", "y", "z", "foo", "bar", "test"];
        let idx: usize = u.arbitrary()?;
        Ok(String8(names[idx % names.len()].to_string()))
    }
}

/// A CSS rule: selector + declarations.
#[derive(Debug, Clone, Arbitrary)]
pub struct CssRule {
    pub selector: Selector,
    pub declarations: Vec<CssDeclaration>,
}

/// A single CSS declaration (property: value).
#[derive(Debug, Clone, Arbitrary)]
pub struct CssDeclaration {
    pub property: CssProperty,
    pub important: bool,
}

/// CSS properties relevant to font/layout fuzzing.
#[derive(Debug, Clone, Arbitrary)]
pub enum CssProperty {
    FontFamily(FontFamilyValue),
    FontSize(LengthValue),
    FontWeight(FontWeightValue),
    FontStyle(FontStyleValue),
    FontVariant(FontVariantValue),
    FontStretch(FontStretchValue),
    FontFeatureSettings(FeatureSettingsValue),
    FontVariationSettings(VariationSettingsValue),
    LineHeight(LengthOrNormal),
    LetterSpacing(LengthOrNormal),
    WordSpacing(LengthOrNormal),
    TextTransform(TextTransformValue),
    TextDecoration(TextDecorationValue),
    TextOverflow(TextOverflowValue),
    WhiteSpace(WhiteSpaceValue),
    WordBreak(WordBreakValue),
    OverflowWrap(OverflowWrapValue),
    Display(DisplayValue),
    Position(PositionValue),
    Float(FloatValue),
    Clear(ClearValue),
    Overflow(OverflowValue),
    Visibility(VisibilityValue),
    Opacity(OpacityValue),
    ZIndex(ZIndexValue),
    Width(LengthOrAuto),
    Height(LengthOrAuto),
    MaxWidth(LengthOrNone),
    MaxHeight(LengthOrNone),
    Margin(LengthOrAuto),
    Padding(LengthValue),
    GridTemplateColumns(GridTemplateValue),
    GridTemplateRows(GridTemplateValue),
    FlexDirection(FlexDirectionValue),
    FlexWrap(FlexWrapValue),
    AlignItems(AlignValue),
    JustifyContent(JustifyValue),
    Transform(TransformList),
    Filter(FilterFunction),
    Animation(AnimationDecl),
    Transition(TransitionDecl),
    Contain(ContainValue),
    ContentVisibility(ContentVisibilityValue),
    WillChange(WillChangeValue),
    WritingMode(WritingModeValue),
    Direction(DirectionValue),
    UnicodeBidi(UnicodeBidiValue),
    ColumnCount(ColumnCountValue),
    ColumnWidth(LengthOrAuto),
    BoxSizing(BoxSizingValue),
    TableLayout(TableLayoutValue),
    // Phase 6: new properties
    Color(ColorValue),
    BackgroundColor(ColorValue),
    BackgroundImage(BackgroundImageValue),
    BorderWidth(LengthValue),
    BorderStyle(BorderStyleValue),
    BorderColor(ColorValue),
    BorderRadius(LengthValue),
    ClipPath(ClipPathValue),
    MixBlendMode(BlendModeValue),
    FlexBasis(LengthOrAuto),
    FlexGrow(u8),
    FlexShrink(u8),
    Gap(LengthValue),
    MinWidth(LengthOrAuto),
    MinHeight(LengthOrAuto),
    Top(LengthOrAuto),
    Left(LengthOrAuto),
    Right(LengthOrAuto),
    Bottom(LengthOrAuto),
    Outline(OutlineValue),
    /// Phase 6: global keyword reset on any property.
    GlobalReset(CssPropertyName, GlobalKeyword),
    // Container queries (very buggy in Blink)
    ContainerType(ContainerTypeValue),
    ContainerName(String8),
    // Subgrid (recently implemented, full of bugs)
    GridTemplateColumnsSubgrid,
    GridTemplateRowsSubgrid,
    // Scroll snap (interacts with layout in complex ways)
    ScrollSnapType(ScrollSnapTypeValue),
    ScrollSnapAlign(ScrollSnapAlignValue),
    // aspect-ratio (interacts with grid/flex sizing)
    AspectRatio(AspectRatioValue),
    // text-wrap: balance/pretty (new, different line breaking)
    TextWrap(TextWrapValue),
    // Logical properties (interact with writing-mode dangerously)
    InlineSize(LengthOrAuto),
    BlockSize(LengthOrAuto),
    MarginInline(LengthOrAuto),
    PaddingBlock(LengthValue),
}

// -- Value types --

#[derive(Debug, Clone, Arbitrary)]
pub enum FontFamilyValue {
    Named(String8),
    Serif,
    SansSerif,
    Monospace,
    Cursive,
    Fantasy,
    SystemUi,
    // Phase 9: multi-family fallback chain
    FallbackChain(FontFallbackChain),
}

/// A font-family fallback chain like "FuzzFont", sans-serif, monospace.
#[derive(Debug, Clone)]
pub struct FontFallbackChain(pub Vec<String>);

impl<'a> Arbitrary<'a> for FontFallbackChain {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let families = [
            "FuzzFont", "serif", "sans-serif", "monospace", "cursive",
            "fantasy", "system-ui", "TestSans", "Arial", "BugSerif",
        ];
        let count = u.int_in_range(2..=4)?;
        let mut chain = Vec::with_capacity(count);
        for _ in 0..count {
            let idx: usize = u.arbitrary()?;
            chain.push(families[idx % families.len()].to_string());
        }
        Ok(FontFallbackChain(chain))
    }
}

#[derive(Debug, Clone, Arbitrary)]
pub enum FontWeightValue {
    Normal,
    Bold,
    Bolder,
    Lighter,
    Number(u16),
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FontStyleValue {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FontVariantValue {
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FontStretchValue {
    Normal,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
    Percent(u16),
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FeatureSettingsValue {
    pub features: Vec<FeatureTag>,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FeatureTag {
    pub tag: OpenTypeTag,
    pub value: u8,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum OpenTypeTag {
    Liga,
    Kern,
    Smcp,
    Onum,
    Lnum,
    Tnum,
    Pnum,
    Swsh,
    Calt,
    Dlig,
    Ss01,
    Ss02,
    // Phase 9: 18 more OpenType tags
    Frac,
    Ordn,
    Sups,
    Subs,
    Salt,
    Ss03,
    Ss04,
    Ss05,
    Ss06,
    Ss07,
    Ss08,
    Ss09,
    Ss10,
    Zero,
    Case,
    Cpsp,
    Titl,
    Pcap,
}

impl OpenTypeTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Liga => "liga",
            Self::Kern => "kern",
            Self::Smcp => "smcp",
            Self::Onum => "onum",
            Self::Lnum => "lnum",
            Self::Tnum => "tnum",
            Self::Pnum => "pnum",
            Self::Swsh => "swsh",
            Self::Calt => "calt",
            Self::Dlig => "dlig",
            Self::Ss01 => "ss01",
            Self::Ss02 => "ss02",
            Self::Frac => "frac",
            Self::Ordn => "ordn",
            Self::Sups => "sups",
            Self::Subs => "subs",
            Self::Salt => "salt",
            Self::Ss03 => "ss03",
            Self::Ss04 => "ss04",
            Self::Ss05 => "ss05",
            Self::Ss06 => "ss06",
            Self::Ss07 => "ss07",
            Self::Ss08 => "ss08",
            Self::Ss09 => "ss09",
            Self::Ss10 => "ss10",
            Self::Zero => "zero",
            Self::Case => "case",
            Self::Cpsp => "cpsp",
            Self::Titl => "titl",
            Self::Pcap => "pcap",
        }
    }
}

#[derive(Debug, Clone, Arbitrary)]
pub struct VariationSettingsValue {
    pub axes: Vec<VariationAxis>,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct VariationAxis {
    pub tag: VarAxisTag,
    pub value: f32,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum VarAxisTag {
    Wght,
    Wdth,
    Slnt,
    Ital,
    Opsz,
}

impl VarAxisTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wght => "wght",
            Self::Wdth => "wdth",
            Self::Slnt => "slnt",
            Self::Ital => "ital",
            Self::Opsz => "opsz",
        }
    }
}

#[derive(Debug, Clone, Arbitrary)]
pub enum LengthValue {
    Px(i32),
    Em(i16),
    Rem(i16),
    Percent(i16),
    Vw(i16),
    Vh(i16),
    Zero,
    /// Phase 6: calc() expression.
    Calc(Box<CalcExpression>),
}

/// A simple calc() expression: left op right.
#[derive(Debug, Clone, Arbitrary)]
pub struct CalcExpression {
    pub left: CalcOperand,
    pub op: CalcOp,
    pub right: CalcOperand,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum CalcOperand {
    Px(i32),
    Percent(i16),
    Em(i16),
    Rem(i16),
    Vw(i16),
    Vh(i16),
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum CalcOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum LengthOrAuto {
    Length(LengthValue),
    Auto,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum LengthOrNone {
    Length(LengthValue),
    None,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum LengthOrNormal {
    Length(LengthValue),
    Normal,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TextTransformValue {
    None,
    Capitalize,
    Uppercase,
    Lowercase,
    FullWidth,
    FullSizeKana,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TextDecorationValue {
    None,
    Underline,
    Overline,
    LineThrough,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TextOverflowValue {
    Clip,
    Ellipsis,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum WhiteSpaceValue {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum WordBreakValue {
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum OverflowWrapValue {
    Normal,
    BreakWord,
    Anywhere,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum DisplayValue {
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Table,
    TableRow,
    TableCell,
    None,
    Contents,
    FlowRoot,
    ListItem,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum PositionValue {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FloatValue {
    None,
    Left,
    Right,
    InlineStart,
    InlineEnd,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ClearValue {
    None,
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum OverflowValue {
    Visible,
    Hidden,
    Scroll,
    Auto,
    Clip,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum VisibilityValue {
    Visible,
    Hidden,
    Collapse,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct OpacityValue(pub u8);

#[derive(Debug, Clone, Arbitrary)]
pub enum ZIndexValue {
    Auto,
    Number(i16),
}

#[derive(Debug, Clone, Arbitrary)]
pub enum GridTemplateValue {
    None,
    Tracks(Vec<TrackSize>),
}

#[derive(Debug, Clone, Arbitrary)]
pub enum TrackSize {
    Length(LengthValue),
    Fr(u8),
    MinMax(LengthValue, LengthValue),
    Auto,
    MinContent,
    MaxContent,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FlexDirectionValue {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FlexWrapValue {
    Nowrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum AlignValue {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum JustifyValue {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct TransformList {
    pub transforms: Vec<TransformFunction>,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum TransformFunction {
    Translate(i16, i16),
    TranslateX(i16),
    TranslateY(i16),
    Scale(i16, i16),
    ScaleX(i16),
    ScaleY(i16),
    Rotate(i16),
    SkewX(i16),
    SkewY(i16),
    Matrix(i16, i16, i16, i16, i16, i16),
}

#[derive(Debug, Clone, Arbitrary)]
pub enum FilterFunction {
    Blur(u8),
    Brightness(u8),
    Contrast(u8),
    Grayscale(u8),
    HueRotate(i16),
    Invert(u8),
    Opacity(u8),
    Saturate(u8),
    Sepia(u8),
    None,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct AnimationDecl {
    pub name: AnimationName,
    pub duration_ms: u16,
    pub timing: TimingFunction,
    pub delay_ms: i16,
    pub iteration_count: IterationCount,
    pub direction: AnimationDirection,
    pub fill_mode: FillMode,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum AnimationName {
    Fade,
    Slide,
    Spin,
    Pulse,
    Bounce,
    Custom(String8),
}

impl AnimationName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Fade => "fade",
            Self::Slide => "slide",
            Self::Spin => "spin",
            Self::Pulse => "pulse",
            Self::Bounce => "bounce",
            Self::Custom(s) => &s.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TimingFunction {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    StepStart,
    StepEnd,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum IterationCount {
    Number(u8),
    Infinite,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum FillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct TransitionDecl {
    pub property: TransitionProperty,
    pub duration_ms: u16,
    pub timing: TimingFunction,
    pub delay_ms: i16,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TransitionProperty {
    All,
    Opacity,
    Transform,
    Width,
    Height,
    BackgroundColor,
    Color,
    FontSize,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ContainValue {
    None,
    Strict,
    Content,
    Size,
    Layout,
    Style,
    Paint,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ContentVisibilityValue {
    Visible,
    Auto,
    Hidden,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum WillChangeValue {
    Auto,
    Transform,
    Opacity,
    Contents,
    ScrollPosition,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum WritingModeValue {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum DirectionValue {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum UnicodeBidiValue {
    Normal,
    Embed,
    Isolate,
    BidiOverride,
    IsolateOverride,
    Plaintext,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum ColumnCountValue {
    Auto,
    Number(u8),
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum BoxSizingValue {
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TableLayoutValue {
    Auto,
    Fixed,
}

// ============================
// Phase 1: @keyframes types
// ============================

/// A @keyframes rule.
#[derive(Debug, Clone, Arbitrary)]
pub struct KeyframesRule {
    pub name: AnimationName,
    pub keyframes: Vec<Keyframe>,
}

/// A single keyframe stop.
#[derive(Debug, Clone, Arbitrary)]
pub struct Keyframe {
    pub offset: KeyframeOffset,
    pub declarations: Vec<CssDeclaration>,
}

/// Keyframe offset: from, to, or percentage.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum KeyframeOffset {
    From,
    To,
    Percent(u8),
}

// ============================
// Phase 6: new CSS value types
// ============================

/// CSS color value.
#[derive(Debug, Clone, Arbitrary)]
pub enum ColorValue {
    Named(NamedColor),
    Hex(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
    CurrentColor,
    Transparent,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum NamedColor {
    Red,
    Blue,
    Green,
    Black,
    White,
    Yellow,
    Cyan,
    Magenta,
    Orange,
    Purple,
    Pink,
    Gray,
}

impl NamedColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Black => "black",
            Self::White => "white",
            Self::Yellow => "yellow",
            Self::Cyan => "cyan",
            Self::Magenta => "magenta",
            Self::Orange => "orange",
            Self::Purple => "purple",
            Self::Pink => "pink",
            Self::Gray => "gray",
        }
    }
}

/// Background image value (gradients).
#[derive(Debug, Clone, Arbitrary)]
pub enum BackgroundImageValue {
    None,
    LinearGradient(GradientDirection, Vec<GradientStop>),
    RadialGradient(Vec<GradientStop>),
    ConicGradient(Vec<GradientStop>),
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum GradientDirection {
    ToTop,
    ToRight,
    ToBottom,
    ToLeft,
    ToTopRight,
    ToBottomRight,
    Deg(i16),
}

#[derive(Debug, Clone, Arbitrary)]
pub struct GradientStop {
    pub color: ColorValue,
    pub position: Option<u8>, // percentage 0-100
}

/// Border style values.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum BorderStyleValue {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    Hidden,
}

/// Clip-path values.
#[derive(Debug, Clone, Arbitrary)]
pub enum ClipPathValue {
    None,
    Circle(u8),                // radius percentage
    Ellipse(u8, u8),           // rx, ry percentages
    Inset(u8, u8, u8, u8),     // top, right, bottom, left percentages
    Polygon(Vec<(u8, u8)>),    // list of (x%, y%) points
}

/// Mix-blend-mode values.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum BlendModeValue {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

/// Outline shorthand value.
#[derive(Debug, Clone, Arbitrary)]
pub struct OutlineValue {
    pub width: LengthValue,
    pub style: BorderStyleValue,
    pub color: ColorValue,
}

/// Global CSS keywords that can reset any property.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum GlobalKeyword {
    Initial,
    Inherit,
    Unset,
    Revert,
}

/// CSS property names for global keyword resets.
#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum CssPropertyName {
    Display,
    Position,
    Width,
    Height,
    Margin,
    Padding,
    FontSize,
    FontFamily,
    FontWeight,
    Color,
    BackgroundColor,
    Overflow,
    Opacity,
    Transform,
    Visibility,
    ContentVisibility,
    Contain,
}

impl CssPropertyName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::Position => "position",
            Self::Width => "width",
            Self::Height => "height",
            Self::Margin => "margin",
            Self::Padding => "padding",
            Self::FontSize => "font-size",
            Self::FontFamily => "font-family",
            Self::FontWeight => "font-weight",
            Self::Color => "color",
            Self::BackgroundColor => "background-color",
            Self::Overflow => "overflow",
            Self::Opacity => "opacity",
            Self::Transform => "transform",
            Self::Visibility => "visibility",
            Self::ContentVisibility => "content-visibility",
            Self::Contain => "contain",
        }
    }
}

// ============================
// Phase 8: at-rules
// ============================

/// CSS at-rule wrapping other CSS rules.
#[derive(Debug, Clone, Arbitrary)]
pub enum AtRule {
    Media(MediaQuery, Vec<CssRule>),
    Container(ContainerQuery, Vec<CssRule>),
    Layer(String8, Vec<CssRule>),
    Supports(SupportsCondition, Vec<CssRule>),
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum MediaQuery {
    MinWidth(u16),
    MaxWidth(u16),
    Screen,
    Print,
    PrefersColorSchemeDark,
    PrefersReducedMotion,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ContainerQuery {
    MinWidth(u16),
    MaxWidth(u16),
}

#[derive(Debug, Clone, Arbitrary)]
pub enum SupportsCondition {
    Property(CssPropertyName),
    Not(Box<SupportsCondition>),
}

// ============================
// New CSS value types for crash-finding
// ============================

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ContainerTypeValue {
    Normal,
    InlineSize,
    Size,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ScrollSnapTypeValue {
    None,
    XMandatory,
    YMandatory,
    XProximity,
    YProximity,
    BlockMandatory,
    InlineMandatory,
    BothMandatory,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum ScrollSnapAlignValue {
    None,
    Start,
    End,
    Center,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum AspectRatioValue {
    Auto,
    Ratio(u16, u16),
}

#[derive(Debug, Clone, Copy, Arbitrary)]
pub enum TextWrapValue {
    Wrap,
    Nowrap,
    Balance,
    Pretty,
    Stable,
}
