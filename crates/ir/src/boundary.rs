use crate::css::*;

/// Hardcoded boundary values known to trigger bugs historically.
/// These are sampled during generation to increase coverage of edge cases.

pub fn boundary_font_sizes() -> Vec<LengthValue> {
    vec![
        LengthValue::Zero,
        LengthValue::Px(0),
        LengthValue::Px(-1),
        LengthValue::Px(1),
        LengthValue::Px(999999),
        LengthValue::Px(-999999),
        LengthValue::Px(i32::MAX),
        LengthValue::Px(i32::MIN),
        LengthValue::Em(0),
        LengthValue::Em(-1),
        LengthValue::Em(10000),
        LengthValue::Rem(0),
        LengthValue::Percent(0),
        LengthValue::Percent(-100),
        LengthValue::Percent(10000),
    ]
}

pub fn boundary_dimensions() -> Vec<LengthOrAuto> {
    vec![
        LengthOrAuto::Auto,
        LengthOrAuto::Length(LengthValue::Zero),
        LengthOrAuto::Length(LengthValue::Px(0)),
        LengthOrAuto::Length(LengthValue::Px(-1)),
        LengthOrAuto::Length(LengthValue::Px(1)),
        LengthOrAuto::Length(LengthValue::Px(999999)),
        LengthOrAuto::Length(LengthValue::Px(i32::MAX)),
        LengthOrAuto::Length(LengthValue::Percent(0)),
        LengthOrAuto::Length(LengthValue::Percent(-100)),
        LengthOrAuto::Length(LengthValue::Vw(100)),
        LengthOrAuto::Length(LengthValue::Vh(100)),
    ]
}

pub fn boundary_displays() -> Vec<DisplayValue> {
    vec![
        DisplayValue::None,
        DisplayValue::Contents,
        DisplayValue::Block,
        DisplayValue::Inline,
        DisplayValue::InlineBlock,
        DisplayValue::Flex,
        DisplayValue::Grid,
        DisplayValue::Table,
        DisplayValue::TableRow,
        DisplayValue::TableCell,
        DisplayValue::FlowRoot,
    ]
}

pub fn boundary_positions() -> Vec<PositionValue> {
    vec![
        PositionValue::Static,
        PositionValue::Relative,
        PositionValue::Absolute,
        PositionValue::Fixed,
        PositionValue::Sticky,
    ]
}

pub fn boundary_content_visibilities() -> Vec<ContentVisibilityValue> {
    vec![
        ContentVisibilityValue::Visible,
        ContentVisibilityValue::Auto,
        ContentVisibilityValue::Hidden,
    ]
}

pub fn boundary_contains() -> Vec<ContainValue> {
    vec![
        ContainValue::None,
        ContainValue::Strict,
        ContainValue::Content,
        ContainValue::Size,
        ContainValue::Layout,
        ContainValue::Paint,
        ContainValue::Style,
    ]
}

pub fn boundary_font_weights() -> Vec<FontWeightValue> {
    vec![
        FontWeightValue::Normal,
        FontWeightValue::Bold,
        FontWeightValue::Number(0),
        FontWeightValue::Number(1),
        FontWeightValue::Number(100),
        FontWeightValue::Number(400),
        FontWeightValue::Number(700),
        FontWeightValue::Number(900),
        FontWeightValue::Number(999),
        FontWeightValue::Number(1000),
        FontWeightValue::Number(u16::MAX),
    ]
}

pub fn boundary_z_indices() -> Vec<ZIndexValue> {
    vec![
        ZIndexValue::Auto,
        ZIndexValue::Number(0),
        ZIndexValue::Number(-1),
        ZIndexValue::Number(1),
        ZIndexValue::Number(i16::MAX),
        ZIndexValue::Number(i16::MIN),
        ZIndexValue::Number(9999),
        ZIndexValue::Number(-9999),
    ]
}

pub fn boundary_opacities() -> Vec<OpacityValue> {
    vec![
        OpacityValue(0),
        OpacityValue(1),
        OpacityValue(50),
        OpacityValue(100),
        OpacityValue(255),
    ]
}

pub fn boundary_overflows() -> Vec<OverflowValue> {
    vec![
        OverflowValue::Visible,
        OverflowValue::Hidden,
        OverflowValue::Scroll,
        OverflowValue::Auto,
        OverflowValue::Clip,
    ]
}

pub fn boundary_will_changes() -> Vec<WillChangeValue> {
    vec![
        WillChangeValue::Auto,
        WillChangeValue::Transform,
        WillChangeValue::Opacity,
        WillChangeValue::Contents,
        WillChangeValue::ScrollPosition,
    ]
}

// Phase 6: new boundary values

pub fn boundary_border_widths() -> Vec<LengthValue> {
    vec![
        LengthValue::Zero,
        LengthValue::Px(0),
        LengthValue::Px(1),
        LengthValue::Px(-1),
        LengthValue::Px(999999),
        LengthValue::Px(i32::MAX),
        LengthValue::Em(0),
        LengthValue::Em(100),
    ]
}

pub fn boundary_border_radii() -> Vec<LengthValue> {
    vec![
        LengthValue::Zero,
        LengthValue::Px(0),
        LengthValue::Px(1),
        LengthValue::Px(999999),
        LengthValue::Percent(0),
        LengthValue::Percent(50),
        LengthValue::Percent(100),
        LengthValue::Percent(10000),
    ]
}

pub fn boundary_colors() -> Vec<ColorValue> {
    vec![
        ColorValue::Transparent,
        ColorValue::CurrentColor,
        ColorValue::Named(NamedColor::Black),
        ColorValue::Named(NamedColor::White),
        ColorValue::Named(NamedColor::Red),
        ColorValue::Hex(0, 0, 0),
        ColorValue::Hex(255, 255, 255),
        ColorValue::Rgba(0, 0, 0, 0),
        ColorValue::Rgba(255, 255, 255, 255),
        ColorValue::Rgba(128, 128, 128, 128),
    ]
}

pub fn boundary_blend_modes() -> Vec<BlendModeValue> {
    vec![
        BlendModeValue::Normal,
        BlendModeValue::Multiply,
        BlendModeValue::Screen,
        BlendModeValue::Overlay,
        BlendModeValue::Difference,
    ]
}

pub fn boundary_border_styles() -> Vec<BorderStyleValue> {
    vec![
        BorderStyleValue::None,
        BorderStyleValue::Solid,
        BorderStyleValue::Dashed,
        BorderStyleValue::Dotted,
        BorderStyleValue::Double,
        BorderStyleValue::Hidden,
    ]
}

pub fn boundary_global_keywords() -> Vec<GlobalKeyword> {
    vec![
        GlobalKeyword::Initial,
        GlobalKeyword::Inherit,
        GlobalKeyword::Unset,
        GlobalKeyword::Revert,
    ]
}
