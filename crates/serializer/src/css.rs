use std::fmt::Write;

use oxiom_ir::css::*;
use oxiom_ir::font;
use oxiom_ir::font::*;

pub fn serialize_css_rules(rules: &[CssRule]) -> String {
    let mut out = String::new();
    for rule in rules {
        serialize_css_rule(rule, &mut out);
        out.push('\n');
    }
    out
}

pub fn serialize_css_rule(rule: &CssRule, out: &mut String) {
    serialize_selector(&rule.selector, out);
    out.push_str(" {\n");
    for decl in &rule.declarations {
        out.push_str("  ");
        serialize_declaration(decl, out);
        out.push('\n');
    }
    out.push('}');
}

pub fn serialize_selector(sel: &Selector, out: &mut String) {
    match sel {
        Selector::Tag(t) => out.push_str(t.as_str()),
        Selector::Class(s) => {
            out.push('.');
            out.push_str(&s.0);
        }
        Selector::Id(node_ref) => {
            write!(out, "#n{}", node_ref.0).unwrap();
        }
        Selector::Pseudo(p) => serialize_pseudo(p, out),
        Selector::Combinator(c) => {
            serialize_selector(&c.left, out);
            out.push_str(c.combinator.as_str());
            serialize_selector(&c.right, out);
        }
        Selector::Universal => out.push('*'),
        // Phase 8: advanced selectors
        Selector::Has(inner) => {
            out.push_str(":has(");
            serialize_selector(inner, out);
            out.push(')');
        }
        Selector::Not(inner) => {
            out.push_str(":not(");
            serialize_selector(inner, out);
            out.push(')');
        }
        Selector::Is(inner) => {
            out.push_str(":is(");
            serialize_selector(inner, out);
            out.push(')');
        }
        Selector::Where(inner) => {
            out.push_str(":where(");
            serialize_selector(inner, out);
            out.push(')');
        }
        Selector::Attribute(attr_sel) => {
            out.push('[');
            out.push_str(attr_sel.attr.as_str());
            if let Some(op) = &attr_sel.op {
                match op.kind {
                    AttrSelectorOpKind::Equals => out.push('='),
                    AttrSelectorOpKind::Contains => out.push_str("*="),
                    AttrSelectorOpKind::StartsWith => out.push_str("^="),
                }
                write!(out, "\"{}\"", op.value.0).unwrap();
            }
            out.push(']');
        }
    }
}

fn serialize_pseudo(p: &PseudoSelector, out: &mut String) {
    match p {
        PseudoSelector::Hover => out.push_str(":hover"),
        PseudoSelector::Focus => out.push_str(":focus"),
        PseudoSelector::Active => out.push_str(":active"),
        PseudoSelector::FirstChild => out.push_str(":first-child"),
        PseudoSelector::LastChild => out.push_str(":last-child"),
        PseudoSelector::NthChild(n) => write!(out, ":nth-child({})", n).unwrap(),
        PseudoSelector::Before => out.push_str("::before"),
        PseudoSelector::After => out.push_str("::after"),
        PseudoSelector::Empty => out.push_str(":empty"),
        PseudoSelector::Root => out.push_str(":root"),
        PseudoSelector::FocusWithin => out.push_str(":focus-within"),
    }
}

pub fn serialize_declaration(decl: &CssDeclaration, out: &mut String) {
    serialize_property(&decl.property, out);
    if decl.important {
        let len = out.len();
        if out.ends_with(';') {
            out.truncate(len - 1);
            out.push_str(" !important;");
        }
    }
}

pub fn serialize_property(prop: &CssProperty, out: &mut String) {
    match prop {
        CssProperty::FontFamily(v) => {
            out.push_str("font-family: ");
            serialize_font_family_value(v, out);
            out.push(';');
        }
        CssProperty::FontSize(v) => {
            out.push_str("font-size: ");
            serialize_length(v, out);
            out.push(';');
        }
        CssProperty::FontWeight(v) => {
            out.push_str("font-weight: ");
            serialize_font_weight(v, out);
            out.push(';');
        }
        CssProperty::FontStyle(v) => {
            out.push_str("font-style: ");
            serialize_font_style(v, out);
            out.push(';');
        }
        CssProperty::FontVariant(v) => {
            out.push_str("font-variant: ");
            serialize_font_variant(v, out);
            out.push(';');
        }
        CssProperty::FontStretch(v) => {
            out.push_str("font-stretch: ");
            serialize_font_stretch(v, out);
            out.push(';');
        }
        CssProperty::FontFeatureSettings(v) => {
            out.push_str("font-feature-settings: ");
            serialize_feature_settings(v, out);
            out.push(';');
        }
        CssProperty::FontVariationSettings(v) => {
            out.push_str("font-variation-settings: ");
            serialize_variation_settings(v, out);
            out.push(';');
        }
        CssProperty::LineHeight(v) => {
            out.push_str("line-height: ");
            serialize_length_or_normal(v, out);
            out.push(';');
        }
        CssProperty::LetterSpacing(v) => {
            out.push_str("letter-spacing: ");
            serialize_length_or_normal(v, out);
            out.push(';');
        }
        CssProperty::WordSpacing(v) => {
            out.push_str("word-spacing: ");
            serialize_length_or_normal(v, out);
            out.push(';');
        }
        CssProperty::TextTransform(v) => {
            out.push_str("text-transform: ");
            out.push_str(match v {
                TextTransformValue::None => "none",
                TextTransformValue::Capitalize => "capitalize",
                TextTransformValue::Uppercase => "uppercase",
                TextTransformValue::Lowercase => "lowercase",
                TextTransformValue::FullWidth => "full-width",
                TextTransformValue::FullSizeKana => "full-size-kana",
            });
            out.push(';');
        }
        CssProperty::TextDecoration(v) => {
            out.push_str("text-decoration: ");
            out.push_str(match v {
                TextDecorationValue::None => "none",
                TextDecorationValue::Underline => "underline",
                TextDecorationValue::Overline => "overline",
                TextDecorationValue::LineThrough => "line-through",
            });
            out.push(';');
        }
        CssProperty::TextOverflow(v) => {
            out.push_str("text-overflow: ");
            out.push_str(match v {
                TextOverflowValue::Clip => "clip",
                TextOverflowValue::Ellipsis => "ellipsis",
            });
            out.push(';');
        }
        CssProperty::WhiteSpace(v) => {
            out.push_str("white-space: ");
            out.push_str(match v {
                WhiteSpaceValue::Normal => "normal",
                WhiteSpaceValue::Nowrap => "nowrap",
                WhiteSpaceValue::Pre => "pre",
                WhiteSpaceValue::PreWrap => "pre-wrap",
                WhiteSpaceValue::PreLine => "pre-line",
                WhiteSpaceValue::BreakSpaces => "break-spaces",
            });
            out.push(';');
        }
        CssProperty::WordBreak(v) => {
            out.push_str("word-break: ");
            out.push_str(match v {
                WordBreakValue::Normal => "normal",
                WordBreakValue::BreakAll => "break-all",
                WordBreakValue::KeepAll => "keep-all",
                WordBreakValue::BreakWord => "break-word",
            });
            out.push(';');
        }
        CssProperty::OverflowWrap(v) => {
            out.push_str("overflow-wrap: ");
            out.push_str(match v {
                OverflowWrapValue::Normal => "normal",
                OverflowWrapValue::BreakWord => "break-word",
                OverflowWrapValue::Anywhere => "anywhere",
            });
            out.push(';');
        }
        CssProperty::Display(v) => {
            out.push_str("display: ");
            serialize_display(v, out);
            out.push(';');
        }
        CssProperty::Position(v) => {
            out.push_str("position: ");
            serialize_position(v, out);
            out.push(';');
        }
        CssProperty::Float(v) => {
            out.push_str("float: ");
            out.push_str(match v {
                FloatValue::None => "none",
                FloatValue::Left => "left",
                FloatValue::Right => "right",
                FloatValue::InlineStart => "inline-start",
                FloatValue::InlineEnd => "inline-end",
            });
            out.push(';');
        }
        CssProperty::Clear(v) => {
            out.push_str("clear: ");
            out.push_str(match v {
                ClearValue::None => "none",
                ClearValue::Left => "left",
                ClearValue::Right => "right",
                ClearValue::Both => "both",
            });
            out.push(';');
        }
        CssProperty::Overflow(v) => {
            out.push_str("overflow: ");
            serialize_overflow(v, out);
            out.push(';');
        }
        CssProperty::Visibility(v) => {
            out.push_str("visibility: ");
            out.push_str(match v {
                VisibilityValue::Visible => "visible",
                VisibilityValue::Hidden => "hidden",
                VisibilityValue::Collapse => "collapse",
            });
            out.push(';');
        }
        CssProperty::Opacity(v) => {
            write!(out, "opacity: {};", v.0 as f32 / 255.0).unwrap();
        }
        CssProperty::ZIndex(v) => {
            out.push_str("z-index: ");
            match v {
                ZIndexValue::Auto => out.push_str("auto"),
                ZIndexValue::Number(n) => write!(out, "{}", n).unwrap(),
            }
            out.push(';');
        }
        CssProperty::Width(v) => {
            out.push_str("width: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::Height(v) => {
            out.push_str("height: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::MaxWidth(v) => {
            out.push_str("max-width: ");
            serialize_length_or_none(v, out);
            out.push(';');
        }
        CssProperty::MaxHeight(v) => {
            out.push_str("max-height: ");
            serialize_length_or_none(v, out);
            out.push(';');
        }
        CssProperty::Margin(v) => {
            out.push_str("margin: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::Padding(v) => {
            out.push_str("padding: ");
            serialize_length(v, out);
            out.push(';');
        }
        CssProperty::GridTemplateColumns(v) => {
            out.push_str("grid-template-columns: ");
            serialize_grid_template(v, out);
            out.push(';');
        }
        CssProperty::GridTemplateRows(v) => {
            out.push_str("grid-template-rows: ");
            serialize_grid_template(v, out);
            out.push(';');
        }
        CssProperty::FlexDirection(v) => {
            out.push_str("flex-direction: ");
            out.push_str(match v {
                FlexDirectionValue::Row => "row",
                FlexDirectionValue::RowReverse => "row-reverse",
                FlexDirectionValue::Column => "column",
                FlexDirectionValue::ColumnReverse => "column-reverse",
            });
            out.push(';');
        }
        CssProperty::FlexWrap(v) => {
            out.push_str("flex-wrap: ");
            out.push_str(match v {
                FlexWrapValue::Nowrap => "nowrap",
                FlexWrapValue::Wrap => "wrap",
                FlexWrapValue::WrapReverse => "wrap-reverse",
            });
            out.push(';');
        }
        CssProperty::AlignItems(v) => {
            out.push_str("align-items: ");
            out.push_str(match v {
                AlignValue::FlexStart => "flex-start",
                AlignValue::FlexEnd => "flex-end",
                AlignValue::Center => "center",
                AlignValue::Baseline => "baseline",
                AlignValue::Stretch => "stretch",
            });
            out.push(';');
        }
        CssProperty::JustifyContent(v) => {
            out.push_str("justify-content: ");
            out.push_str(match v {
                JustifyValue::FlexStart => "flex-start",
                JustifyValue::FlexEnd => "flex-end",
                JustifyValue::Center => "center",
                JustifyValue::SpaceBetween => "space-between",
                JustifyValue::SpaceAround => "space-around",
                JustifyValue::SpaceEvenly => "space-evenly",
            });
            out.push(';');
        }
        CssProperty::Transform(v) => {
            out.push_str("transform: ");
            serialize_transform_list(v, out);
            out.push(';');
        }
        CssProperty::Filter(v) => {
            out.push_str("filter: ");
            serialize_filter(v, out);
            out.push(';');
        }
        CssProperty::Animation(v) => {
            out.push_str("animation: ");
            serialize_animation(v, out);
            out.push(';');
        }
        CssProperty::Transition(v) => {
            out.push_str("transition: ");
            serialize_transition(v, out);
            out.push(';');
        }
        CssProperty::Contain(v) => {
            out.push_str("contain: ");
            out.push_str(match v {
                ContainValue::None => "none",
                ContainValue::Strict => "strict",
                ContainValue::Content => "content",
                ContainValue::Size => "size",
                ContainValue::Layout => "layout",
                ContainValue::Style => "style",
                ContainValue::Paint => "paint",
            });
            out.push(';');
        }
        CssProperty::ContentVisibility(v) => {
            out.push_str("content-visibility: ");
            out.push_str(match v {
                ContentVisibilityValue::Visible => "visible",
                ContentVisibilityValue::Auto => "auto",
                ContentVisibilityValue::Hidden => "hidden",
            });
            out.push(';');
        }
        CssProperty::WillChange(v) => {
            out.push_str("will-change: ");
            out.push_str(match v {
                WillChangeValue::Auto => "auto",
                WillChangeValue::Transform => "transform",
                WillChangeValue::Opacity => "opacity",
                WillChangeValue::Contents => "contents",
                WillChangeValue::ScrollPosition => "scroll-position",
            });
            out.push(';');
        }
        CssProperty::WritingMode(v) => {
            out.push_str("writing-mode: ");
            out.push_str(match v {
                WritingModeValue::HorizontalTb => "horizontal-tb",
                WritingModeValue::VerticalRl => "vertical-rl",
                WritingModeValue::VerticalLr => "vertical-lr",
                WritingModeValue::SidewaysRl => "sideways-rl",
                WritingModeValue::SidewaysLr => "sideways-lr",
            });
            out.push(';');
        }
        CssProperty::Direction(v) => {
            out.push_str("direction: ");
            out.push_str(match v {
                DirectionValue::Ltr => "ltr",
                DirectionValue::Rtl => "rtl",
            });
            out.push(';');
        }
        CssProperty::UnicodeBidi(v) => {
            out.push_str("unicode-bidi: ");
            out.push_str(match v {
                UnicodeBidiValue::Normal => "normal",
                UnicodeBidiValue::Embed => "embed",
                UnicodeBidiValue::Isolate => "isolate",
                UnicodeBidiValue::BidiOverride => "bidi-override",
                UnicodeBidiValue::IsolateOverride => "isolate-override",
                UnicodeBidiValue::Plaintext => "plaintext",
            });
            out.push(';');
        }
        CssProperty::ColumnCount(v) => {
            out.push_str("column-count: ");
            match v {
                ColumnCountValue::Auto => out.push_str("auto"),
                ColumnCountValue::Number(n) => write!(out, "{}", n).unwrap(),
            }
            out.push(';');
        }
        CssProperty::ColumnWidth(v) => {
            out.push_str("column-width: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::BoxSizing(v) => {
            out.push_str("box-sizing: ");
            out.push_str(match v {
                BoxSizingValue::ContentBox => "content-box",
                BoxSizingValue::BorderBox => "border-box",
            });
            out.push(';');
        }
        CssProperty::TableLayout(v) => {
            out.push_str("table-layout: ");
            out.push_str(match v {
                TableLayoutValue::Auto => "auto",
                TableLayoutValue::Fixed => "fixed",
            });
            out.push(';');
        }
        // Phase 6: new properties
        CssProperty::Color(v) => {
            out.push_str("color: ");
            serialize_color(v, out);
            out.push(';');
        }
        CssProperty::BackgroundColor(v) => {
            out.push_str("background-color: ");
            serialize_color(v, out);
            out.push(';');
        }
        CssProperty::BackgroundImage(v) => {
            out.push_str("background-image: ");
            serialize_background_image(v, out);
            out.push(';');
        }
        CssProperty::BorderWidth(v) => {
            out.push_str("border-width: ");
            serialize_length(v, out);
            out.push(';');
        }
        CssProperty::BorderStyle(v) => {
            out.push_str("border-style: ");
            serialize_border_style(v, out);
            out.push(';');
        }
        CssProperty::BorderColor(v) => {
            out.push_str("border-color: ");
            serialize_color(v, out);
            out.push(';');
        }
        CssProperty::BorderRadius(v) => {
            out.push_str("border-radius: ");
            serialize_length(v, out);
            out.push(';');
        }
        CssProperty::ClipPath(v) => {
            out.push_str("clip-path: ");
            serialize_clip_path(v, out);
            out.push(';');
        }
        CssProperty::MixBlendMode(v) => {
            out.push_str("mix-blend-mode: ");
            serialize_blend_mode(v, out);
            out.push(';');
        }
        CssProperty::FlexBasis(v) => {
            out.push_str("flex-basis: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::FlexGrow(v) => {
            write!(out, "flex-grow: {};", v).unwrap();
        }
        CssProperty::FlexShrink(v) => {
            write!(out, "flex-shrink: {};", v).unwrap();
        }
        CssProperty::Gap(v) => {
            out.push_str("gap: ");
            serialize_length(v, out);
            out.push(';');
        }
        CssProperty::MinWidth(v) => {
            out.push_str("min-width: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::MinHeight(v) => {
            out.push_str("min-height: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::Top(v) => {
            out.push_str("top: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::Left(v) => {
            out.push_str("left: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::Right(v) => {
            out.push_str("right: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::Bottom(v) => {
            out.push_str("bottom: ");
            serialize_length_or_auto(v, out);
            out.push(';');
        }
        CssProperty::Outline(v) => {
            out.push_str("outline: ");
            serialize_length(&v.width, out);
            out.push(' ');
            serialize_border_style(&v.style, out);
            out.push(' ');
            serialize_color(&v.color, out);
            out.push(';');
        }
        CssProperty::GlobalReset(prop_name, keyword) => {
            out.push_str(prop_name.as_str());
            out.push_str(": ");
            out.push_str(match keyword {
                GlobalKeyword::Initial => "initial",
                GlobalKeyword::Inherit => "inherit",
                GlobalKeyword::Unset => "unset",
                GlobalKeyword::Revert => "revert",
            });
            out.push(';');
        }
    }
}

pub fn serialize_length(v: &LengthValue, out: &mut String) {
    match v {
        LengthValue::Px(n) => write!(out, "{}px", n).unwrap(),
        LengthValue::Em(n) => write!(out, "{}em", n).unwrap(),
        LengthValue::Rem(n) => write!(out, "{}rem", n).unwrap(),
        LengthValue::Percent(n) => write!(out, "{}%", n).unwrap(),
        LengthValue::Vw(n) => write!(out, "{}vw", n).unwrap(),
        LengthValue::Vh(n) => write!(out, "{}vh", n).unwrap(),
        LengthValue::Zero => out.push('0'),
        LengthValue::Calc(expr) => {
            out.push_str("calc(");
            serialize_calc_operand(&expr.left, out);
            match expr.op {
                CalcOp::Add => out.push_str(" + "),
                CalcOp::Sub => out.push_str(" - "),
                CalcOp::Mul => out.push_str(" * "),
                CalcOp::Div => out.push_str(" / "),
            }
            serialize_calc_operand(&expr.right, out);
            out.push(')');
        }
    }
}

fn serialize_calc_operand(v: &CalcOperand, out: &mut String) {
    match v {
        CalcOperand::Px(n) => write!(out, "{}px", n).unwrap(),
        CalcOperand::Percent(n) => write!(out, "{}%", n).unwrap(),
        CalcOperand::Em(n) => write!(out, "{}em", n).unwrap(),
        CalcOperand::Rem(n) => write!(out, "{}rem", n).unwrap(),
        CalcOperand::Vw(n) => write!(out, "{}vw", n).unwrap(),
        CalcOperand::Vh(n) => write!(out, "{}vh", n).unwrap(),
    }
}

fn serialize_length_or_auto(v: &LengthOrAuto, out: &mut String) {
    match v {
        LengthOrAuto::Length(l) => serialize_length(l, out),
        LengthOrAuto::Auto => out.push_str("auto"),
    }
}

fn serialize_length_or_none(v: &LengthOrNone, out: &mut String) {
    match v {
        LengthOrNone::Length(l) => serialize_length(l, out),
        LengthOrNone::None => out.push_str("none"),
    }
}

fn serialize_length_or_normal(v: &LengthOrNormal, out: &mut String) {
    match v {
        LengthOrNormal::Length(l) => serialize_length(l, out),
        LengthOrNormal::Normal => out.push_str("normal"),
    }
}

fn serialize_font_family_value(v: &FontFamilyValue, out: &mut String) {
    match v {
        FontFamilyValue::Named(s) => write!(out, "\"{}\"", s.0).unwrap(),
        FontFamilyValue::Serif => out.push_str("serif"),
        FontFamilyValue::SansSerif => out.push_str("sans-serif"),
        FontFamilyValue::Monospace => out.push_str("monospace"),
        FontFamilyValue::Cursive => out.push_str("cursive"),
        FontFamilyValue::Fantasy => out.push_str("fantasy"),
        FontFamilyValue::SystemUi => out.push_str("system-ui"),
        FontFamilyValue::FallbackChain(chain) => {
            for (i, family) in chain.0.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                let generic = matches!(
                    family.as_str(),
                    "serif" | "sans-serif" | "monospace" | "cursive" | "fantasy" | "system-ui"
                );
                if generic {
                    out.push_str(family);
                } else {
                    write!(out, "\"{}\"", family).unwrap();
                }
            }
        }
    }
}

fn serialize_font_weight(v: &FontWeightValue, out: &mut String) {
    match v {
        FontWeightValue::Normal => out.push_str("normal"),
        FontWeightValue::Bold => out.push_str("bold"),
        FontWeightValue::Bolder => out.push_str("bolder"),
        FontWeightValue::Lighter => out.push_str("lighter"),
        FontWeightValue::Number(n) => write!(out, "{}", n).unwrap(),
    }
}

fn serialize_font_style(v: &FontStyleValue, out: &mut String) {
    match v {
        FontStyleValue::Normal => out.push_str("normal"),
        FontStyleValue::Italic => out.push_str("italic"),
        FontStyleValue::Oblique => out.push_str("oblique"),
    }
}

fn serialize_font_variant(v: &FontVariantValue, out: &mut String) {
    out.push_str(match v {
        FontVariantValue::Normal => "normal",
        FontVariantValue::SmallCaps => "small-caps",
        FontVariantValue::AllSmallCaps => "all-small-caps",
        FontVariantValue::PetiteCaps => "petite-caps",
        FontVariantValue::AllPetiteCaps => "all-petite-caps",
        FontVariantValue::Unicase => "unicase",
        FontVariantValue::TitlingCaps => "titling-caps",
    });
}

fn serialize_font_stretch(v: &FontStretchValue, out: &mut String) {
    match v {
        FontStretchValue::Normal => out.push_str("normal"),
        FontStretchValue::UltraCondensed => out.push_str("ultra-condensed"),
        FontStretchValue::ExtraCondensed => out.push_str("extra-condensed"),
        FontStretchValue::Condensed => out.push_str("condensed"),
        FontStretchValue::SemiCondensed => out.push_str("semi-condensed"),
        FontStretchValue::SemiExpanded => out.push_str("semi-expanded"),
        FontStretchValue::Expanded => out.push_str("expanded"),
        FontStretchValue::ExtraExpanded => out.push_str("extra-expanded"),
        FontStretchValue::UltraExpanded => out.push_str("ultra-expanded"),
        FontStretchValue::Percent(n) => write!(out, "{}%", n).unwrap(),
    }
}

fn serialize_feature_settings(v: &FeatureSettingsValue, out: &mut String) {
    if v.features.is_empty() {
        out.push_str("normal");
        return;
    }
    for (i, f) in v.features.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write!(out, "\"{}\" {}", f.tag.as_str(), f.value).unwrap();
    }
}

fn serialize_variation_settings(v: &VariationSettingsValue, out: &mut String) {
    if v.axes.is_empty() {
        out.push_str("normal");
        return;
    }
    for (i, a) in v.axes.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let val = if a.value.is_finite() { a.value } else { 0.0 };
        write!(out, "\"{}\" {}", a.tag.as_str(), val).unwrap();
    }
}

fn serialize_display(v: &DisplayValue, out: &mut String) {
    out.push_str(match v {
        DisplayValue::Block => "block",
        DisplayValue::Inline => "inline",
        DisplayValue::InlineBlock => "inline-block",
        DisplayValue::Flex => "flex",
        DisplayValue::InlineFlex => "inline-flex",
        DisplayValue::Grid => "grid",
        DisplayValue::InlineGrid => "inline-grid",
        DisplayValue::Table => "table",
        DisplayValue::TableRow => "table-row",
        DisplayValue::TableCell => "table-cell",
        DisplayValue::None => "none",
        DisplayValue::Contents => "contents",
        DisplayValue::FlowRoot => "flow-root",
        DisplayValue::ListItem => "list-item",
    });
}

fn serialize_position(v: &PositionValue, out: &mut String) {
    out.push_str(match v {
        PositionValue::Static => "static",
        PositionValue::Relative => "relative",
        PositionValue::Absolute => "absolute",
        PositionValue::Fixed => "fixed",
        PositionValue::Sticky => "sticky",
    });
}

fn serialize_overflow(v: &OverflowValue, out: &mut String) {
    out.push_str(match v {
        OverflowValue::Visible => "visible",
        OverflowValue::Hidden => "hidden",
        OverflowValue::Scroll => "scroll",
        OverflowValue::Auto => "auto",
        OverflowValue::Clip => "clip",
    });
}

fn serialize_grid_template(v: &GridTemplateValue, out: &mut String) {
    match v {
        GridTemplateValue::None => out.push_str("none"),
        GridTemplateValue::Tracks(tracks) => {
            if tracks.is_empty() {
                out.push_str("none");
                return;
            }
            for (i, t) in tracks.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                match t {
                    TrackSize::Length(l) => serialize_length(l, out),
                    TrackSize::Fr(n) => write!(out, "{}fr", n).unwrap(),
                    TrackSize::MinMax(a, b) => {
                        out.push_str("minmax(");
                        serialize_length(a, out);
                        out.push_str(", ");
                        serialize_length(b, out);
                        out.push(')');
                    }
                    TrackSize::Auto => out.push_str("auto"),
                    TrackSize::MinContent => out.push_str("min-content"),
                    TrackSize::MaxContent => out.push_str("max-content"),
                }
            }
        }
    }
}

fn serialize_transform_list(v: &TransformList, out: &mut String) {
    if v.transforms.is_empty() {
        out.push_str("none");
        return;
    }
    for (i, t) in v.transforms.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        match t {
            TransformFunction::Translate(x, y) => write!(out, "translate({}px, {}px)", x, y).unwrap(),
            TransformFunction::TranslateX(x) => write!(out, "translateX({}px)", x).unwrap(),
            TransformFunction::TranslateY(y) => write!(out, "translateY({}px)", y).unwrap(),
            TransformFunction::Scale(x, y) => {
                write!(out, "scale({}, {})", *x as f32 / 100.0, *y as f32 / 100.0).unwrap()
            }
            TransformFunction::ScaleX(x) => write!(out, "scaleX({})", *x as f32 / 100.0).unwrap(),
            TransformFunction::ScaleY(y) => write!(out, "scaleY({})", *y as f32 / 100.0).unwrap(),
            TransformFunction::Rotate(d) => write!(out, "rotate({}deg)", d).unwrap(),
            TransformFunction::SkewX(d) => write!(out, "skewX({}deg)", d).unwrap(),
            TransformFunction::SkewY(d) => write!(out, "skewY({}deg)", d).unwrap(),
            TransformFunction::Matrix(a, b, c, d, e, f) => {
                write!(out, "matrix({}, {}, {}, {}, {}, {})", a, b, c, d, e, f).unwrap()
            }
        }
    }
}

fn serialize_filter(v: &FilterFunction, out: &mut String) {
    match v {
        FilterFunction::Blur(n) => write!(out, "blur({}px)", n).unwrap(),
        FilterFunction::Brightness(n) => write!(out, "brightness({}%)", n).unwrap(),
        FilterFunction::Contrast(n) => write!(out, "contrast({}%)", n).unwrap(),
        FilterFunction::Grayscale(n) => write!(out, "grayscale({}%)", n).unwrap(),
        FilterFunction::HueRotate(n) => write!(out, "hue-rotate({}deg)", n).unwrap(),
        FilterFunction::Invert(n) => write!(out, "invert({}%)", n).unwrap(),
        FilterFunction::Opacity(n) => write!(out, "opacity({}%)", n).unwrap(),
        FilterFunction::Saturate(n) => write!(out, "saturate({}%)", n).unwrap(),
        FilterFunction::Sepia(n) => write!(out, "sepia({}%)", n).unwrap(),
        FilterFunction::None => out.push_str("none"),
    }
}

fn serialize_animation(v: &AnimationDecl, out: &mut String) {
    out.push_str(v.name.as_str());
    write!(out, " {}ms", v.duration_ms).unwrap();
    out.push(' ');
    serialize_timing(&v.timing, out);
    write!(out, " {}ms", v.delay_ms).unwrap();
    out.push(' ');
    match &v.iteration_count {
        IterationCount::Number(n) => write!(out, "{}", n).unwrap(),
        IterationCount::Infinite => out.push_str("infinite"),
    }
    out.push(' ');
    out.push_str(match v.direction {
        AnimationDirection::Normal => "normal",
        AnimationDirection::Reverse => "reverse",
        AnimationDirection::Alternate => "alternate",
        AnimationDirection::AlternateReverse => "alternate-reverse",
    });
    out.push(' ');
    out.push_str(match v.fill_mode {
        FillMode::None => "none",
        FillMode::Forwards => "forwards",
        FillMode::Backwards => "backwards",
        FillMode::Both => "both",
    });
}

fn serialize_transition(v: &TransitionDecl, out: &mut String) {
    out.push_str(match v.property {
        TransitionProperty::All => "all",
        TransitionProperty::Opacity => "opacity",
        TransitionProperty::Transform => "transform",
        TransitionProperty::Width => "width",
        TransitionProperty::Height => "height",
        TransitionProperty::BackgroundColor => "background-color",
        TransitionProperty::Color => "color",
        TransitionProperty::FontSize => "font-size",
    });
    write!(out, " {}ms ", v.duration_ms).unwrap();
    serialize_timing(&v.timing, out);
    write!(out, " {}ms", v.delay_ms).unwrap();
}

fn serialize_timing(v: &TimingFunction, out: &mut String) {
    out.push_str(match v {
        TimingFunction::Ease => "ease",
        TimingFunction::Linear => "linear",
        TimingFunction::EaseIn => "ease-in",
        TimingFunction::EaseOut => "ease-out",
        TimingFunction::EaseInOut => "ease-in-out",
        TimingFunction::StepStart => "step-start",
        TimingFunction::StepEnd => "step-end",
    });
}

// ============================
// Phase 6: new serializers
// ============================

fn serialize_color(v: &ColorValue, out: &mut String) {
    match v {
        ColorValue::Named(n) => out.push_str(n.as_str()),
        ColorValue::Hex(r, g, b) => write!(out, "#{:02x}{:02x}{:02x}", r, g, b).unwrap(),
        ColorValue::Rgba(r, g, b, a) => {
            write!(out, "rgba({}, {}, {}, {})", r, g, b, *a as f32 / 255.0).unwrap()
        }
        ColorValue::CurrentColor => out.push_str("currentColor"),
        ColorValue::Transparent => out.push_str("transparent"),
    }
}

fn serialize_background_image(v: &BackgroundImageValue, out: &mut String) {
    match v {
        BackgroundImageValue::None => out.push_str("none"),
        BackgroundImageValue::LinearGradient(dir, stops) => {
            out.push_str("linear-gradient(");
            serialize_gradient_direction(dir, out);
            for stop in stops {
                out.push_str(", ");
                serialize_color(&stop.color, out);
                if let Some(pos) = stop.position {
                    write!(out, " {}%", pos).unwrap();
                }
            }
            out.push(')');
        }
        BackgroundImageValue::RadialGradient(stops) => {
            out.push_str("radial-gradient(");
            for (i, stop) in stops.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                serialize_color(&stop.color, out);
                if let Some(pos) = stop.position {
                    write!(out, " {}%", pos).unwrap();
                }
            }
            out.push(')');
        }
        BackgroundImageValue::ConicGradient(stops) => {
            out.push_str("conic-gradient(");
            for (i, stop) in stops.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                serialize_color(&stop.color, out);
                if let Some(pos) = stop.position {
                    write!(out, " {}%", pos).unwrap();
                }
            }
            out.push(')');
        }
    }
}

fn serialize_gradient_direction(v: &GradientDirection, out: &mut String) {
    match v {
        GradientDirection::ToTop => out.push_str("to top"),
        GradientDirection::ToRight => out.push_str("to right"),
        GradientDirection::ToBottom => out.push_str("to bottom"),
        GradientDirection::ToLeft => out.push_str("to left"),
        GradientDirection::ToTopRight => out.push_str("to top right"),
        GradientDirection::ToBottomRight => out.push_str("to bottom right"),
        GradientDirection::Deg(d) => write!(out, "{}deg", d).unwrap(),
    }
}

fn serialize_border_style(v: &BorderStyleValue, out: &mut String) {
    out.push_str(match v {
        BorderStyleValue::None => "none",
        BorderStyleValue::Solid => "solid",
        BorderStyleValue::Dashed => "dashed",
        BorderStyleValue::Dotted => "dotted",
        BorderStyleValue::Double => "double",
        BorderStyleValue::Groove => "groove",
        BorderStyleValue::Ridge => "ridge",
        BorderStyleValue::Inset => "inset",
        BorderStyleValue::Outset => "outset",
        BorderStyleValue::Hidden => "hidden",
    });
}

fn serialize_clip_path(v: &ClipPathValue, out: &mut String) {
    match v {
        ClipPathValue::None => out.push_str("none"),
        ClipPathValue::Circle(r) => write!(out, "circle({}%)", r).unwrap(),
        ClipPathValue::Ellipse(rx, ry) => write!(out, "ellipse({}% {}%)", rx, ry).unwrap(),
        ClipPathValue::Inset(t, r, b, l) => {
            write!(out, "inset({}% {}% {}% {}%)", t, r, b, l).unwrap()
        }
        ClipPathValue::Polygon(points) => {
            out.push_str("polygon(");
            for (i, (x, y)) in points.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write!(out, "{}% {}%", x, y).unwrap();
            }
            out.push(')');
        }
    }
}

fn serialize_blend_mode(v: &BlendModeValue, out: &mut String) {
    out.push_str(match v {
        BlendModeValue::Normal => "normal",
        BlendModeValue::Multiply => "multiply",
        BlendModeValue::Screen => "screen",
        BlendModeValue::Overlay => "overlay",
        BlendModeValue::Darken => "darken",
        BlendModeValue::Lighten => "lighten",
        BlendModeValue::ColorDodge => "color-dodge",
        BlendModeValue::ColorBurn => "color-burn",
        BlendModeValue::HardLight => "hard-light",
        BlendModeValue::SoftLight => "soft-light",
        BlendModeValue::Difference => "difference",
        BlendModeValue::Exclusion => "exclusion",
    });
}

// ============================
// Phase 1: @keyframes serialization
// ============================

pub fn serialize_keyframes_rule(rule: &KeyframesRule, out: &mut String) {
    write!(out, "@keyframes {} {{\n", rule.name.as_str()).unwrap();
    for kf in &rule.keyframes {
        out.push_str("  ");
        match kf.offset {
            KeyframeOffset::From => out.push_str("from"),
            KeyframeOffset::To => out.push_str("to"),
            KeyframeOffset::Percent(p) => write!(out, "{}%", p.min(100)).unwrap(),
        }
        out.push_str(" {\n");
        for decl in &kf.declarations {
            out.push_str("    ");
            serialize_declaration(decl, out);
            out.push('\n');
        }
        out.push_str("  }\n");
    }
    out.push('}');
}

// ============================
// Phase 8: at-rule serialization
// ============================

pub fn serialize_at_rule(rule: &AtRule, out: &mut String) {
    match rule {
        AtRule::Media(query, rules) => {
            out.push_str("@media ");
            serialize_media_query(query, out);
            out.push_str(" {\n");
            for r in rules {
                out.push_str("  ");
                serialize_css_rule(r, out);
                out.push('\n');
            }
            out.push('}');
        }
        AtRule::Container(query, rules) => {
            out.push_str("@container ");
            serialize_container_query(query, out);
            out.push_str(" {\n");
            for r in rules {
                out.push_str("  ");
                serialize_css_rule(r, out);
                out.push('\n');
            }
            out.push('}');
        }
        AtRule::Layer(name, rules) => {
            write!(out, "@layer {} {{\n", name.0).unwrap();
            for r in rules {
                out.push_str("  ");
                serialize_css_rule(r, out);
                out.push('\n');
            }
            out.push('}');
        }
        AtRule::Supports(condition, rules) => {
            out.push_str("@supports ");
            serialize_supports_condition(condition, out);
            out.push_str(" {\n");
            for r in rules {
                out.push_str("  ");
                serialize_css_rule(r, out);
                out.push('\n');
            }
            out.push('}');
        }
    }
}

fn serialize_media_query(q: &MediaQuery, out: &mut String) {
    match q {
        MediaQuery::MinWidth(w) => write!(out, "(min-width: {}px)", w).unwrap(),
        MediaQuery::MaxWidth(w) => write!(out, "(max-width: {}px)", w).unwrap(),
        MediaQuery::Screen => out.push_str("screen"),
        MediaQuery::Print => out.push_str("print"),
        MediaQuery::PrefersColorSchemeDark => {
            out.push_str("(prefers-color-scheme: dark)")
        }
        MediaQuery::PrefersReducedMotion => {
            out.push_str("(prefers-reduced-motion: reduce)")
        }
    }
}

fn serialize_container_query(q: &ContainerQuery, out: &mut String) {
    match q {
        ContainerQuery::MinWidth(w) => write!(out, "(min-width: {}px)", w).unwrap(),
        ContainerQuery::MaxWidth(w) => write!(out, "(max-width: {}px)", w).unwrap(),
    }
}

fn serialize_supports_condition(c: &SupportsCondition, out: &mut String) {
    match c {
        SupportsCondition::Property(prop) => {
            write!(out, "({}:initial)", prop.as_str()).unwrap()
        }
        SupportsCondition::Not(inner) => {
            out.push_str("not ");
            serialize_supports_condition(inner, out);
        }
    }
}

// ============================
// @font-face serialization
// ============================

/// Serialize a @font-face declaration to CSS text.
pub fn serialize_font_face(face: &FontFaceDecl, out: &mut String) {
    out.push_str("@font-face {\n");
    write!(out, "  font-family: \"{}\";\n", face.family.0).unwrap();

    if !face.src.is_empty() {
        out.push_str("  src: ");
        for (i, src) in face.src.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            match src {
                FontSrc::Local(name) => write!(out, "local(\"{}\")", name.0).unwrap(),
                FontSrc::Url(url) => {
                    write!(
                        out,
                        "url(\"font.{}\") format(\"{}\")",
                        url.format.extension(),
                        url.format.as_str()
                    )
                    .unwrap();
                }
                FontSrc::DataUrl => {
                    write!(
                        out,
                        "url(\"data:font/woff2;base64,{}\") format(\"woff2\")",
                        font::MINIMAL_WOFF2_BASE64
                    )
                    .unwrap();
                }
            }
        }
        out.push_str(";\n");
    }

    if let Some(w) = &face.weight {
        out.push_str("  font-weight: ");
        write!(out, "{}", w.start.clamp(1, 1000)).unwrap();
        if let Some(end) = w.end {
            write!(out, " {}", end.clamp(1, 1000)).unwrap();
        }
        out.push_str(";\n");
    }

    if let Some(s) = &face.style {
        out.push_str("  font-style: ");
        match s {
            FontStyleRange::Normal => out.push_str("normal"),
            FontStyleRange::Italic => out.push_str("italic"),
            FontStyleRange::Oblique(start, end) => {
                out.push_str("oblique");
                if let Some(s) = start {
                    write!(out, " {}deg", s).unwrap();
                }
                if let Some(e) = end {
                    write!(out, " {}deg", e).unwrap();
                }
            }
        }
        out.push_str(";\n");
    }

    write!(out, "  font-display: {};\n", face.display.as_str()).unwrap();

    if let Some(range) = &face.unicode_range {
        let start = range.start & 0x10FFFF;
        out.push_str("  unicode-range: ");
        if let Some(end) = range.end {
            let end = end & 0x10FFFF;
            write!(out, "U+{:04X}-{:04X}", start, end).unwrap();
        } else {
            write!(out, "U+{:04X}", start).unwrap();
        }
        out.push_str(";\n");
    }

    if !face.feature_settings.is_empty() {
        out.push_str("  font-feature-settings: ");
        for (i, f) in face.feature_settings.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let val = match f.value {
                font::FeatureValue::On => 1,
                font::FeatureValue::Off => 0,
                font::FeatureValue::Number(n) => n as i32,
            };
            write!(out, "\"{}\" {}", f.tag.as_str(), val).unwrap();
        }
        out.push_str(";\n");
    }

    if !face.variation_settings.is_empty() {
        out.push_str("  font-variation-settings: ");
        for (i, v) in face.variation_settings.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let val = if v.value.is_finite() { v.value } else { 0.0 };
            write!(out, "\"{}\" {}", v.tag.as_str(), val).unwrap();
        }
        out.push_str(";\n");
    }

    // Phase 9: font metrics overrides
    if let Some(sa) = face.size_adjust {
        write!(out, "  size-adjust: {}%;\n", sa).unwrap();
    }
    if let Some(ao) = face.ascent_override {
        write!(out, "  ascent-override: {}%;\n", ao).unwrap();
    }
    if let Some(d) = face.descent_override {
        write!(out, "  descent-override: {}%;\n", d).unwrap();
    }
    if let Some(lg) = face.line_gap_override {
        write!(out, "  line-gap-override: {}%;\n", lg).unwrap();
    }

    out.push('}');
}

/// Serialize a property to its CSS property name string (for inline styles in JS).
pub fn property_to_css_value(prop: &CssProperty) -> String {
    let mut out = String::new();
    serialize_property(prop, &mut out);
    out
}
