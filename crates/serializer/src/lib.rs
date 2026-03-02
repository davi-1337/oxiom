pub mod css;
pub mod html;
pub mod js;

use oxiom_ir::css::{AtRule, CssRule, KeyframesRule};
use oxiom_ir::font::FontFaceDecl;
use oxiom_ir::html::DomTree;
use oxiom_ir::js::JsOperation;

/// Serialize a complete fuzz program into an HTML document.
pub fn serialize(
    font_faces: &[FontFaceDecl],
    css_rules: &[CssRule],
    dom: &DomTree,
    script: &[JsOperation],
    keyframes: &[KeyframesRule],
    at_rules: &[AtRule],
) -> String {
    let mut out = String::with_capacity(8192);

    out.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n");

    // Style block
    out.push_str("<style>\n");

    // @font-face declarations
    for face in font_faces {
        css::serialize_font_face(face, &mut out);
        out.push('\n');
    }

    // @keyframes declarations (Phase 1)
    for kf in keyframes {
        css::serialize_keyframes_rule(kf, &mut out);
        out.push('\n');
    }

    // At-rules (Phase 8)
    for at_rule in at_rules {
        css::serialize_at_rule(at_rule, &mut out);
        out.push('\n');
    }

    // CSS rules
    for rule in css_rules {
        css::serialize_css_rule(rule, &mut out);
        out.push('\n');
    }

    out.push_str("</style>\n</head>\n<body>\n");

    // DOM tree
    let (dom_html, _node_count) = html::serialize_dom_tree(dom);
    out.push_str(&dom_html);

    // Script block
    out.push_str("\n<script>\n");
    let js_code = js::serialize_js_operations(script);
    out.push_str(&js_code);
    out.push_str("</script>\n");

    out.push_str("</body>\n</html>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxiom_ir::html::*;

    #[test]
    fn test_empty_program() {
        let html = serialize(
            &[],
            &[],
            &DomTree {
                root_children: vec![],
            },
            &[],
            &[],
            &[],
        );
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html>"));
        assert!(html.contains("</html>"));
        assert!(html.contains("<style>"));
        assert!(html.contains("</style>"));
        assert!(html.contains("<script>"));
        assert!(html.contains("</script>"));
    }

    #[test]
    fn test_dom_serialization() {
        let tree = DomTree {
            root_children: vec![DomNode {
                element: HtmlElement::Div,
                text_content: Some(TextContent("hello".to_string())),
                children: vec![DomNode {
                    element: HtmlElement::Span,
                    text_content: None,
                    children: vec![],
                }],
            }],
        };
        let html = serialize(&[], &[], &tree, &[], &[], &[]);
        assert!(html.contains("<div id=\"n0\">"));
        assert!(html.contains("hello"));
        assert!(html.contains("<span id=\"n1\">"));
        assert!(html.contains("</span>"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_arbitrary_roundtrip() {
        use arbitrary::{Arbitrary, Unstructured};
        use oxiom_generator::FuzzProgram;

        // Generate programs from random bytes with larger buffer and multiple seeds
        for seed_offset in 0..4u8 {
            let bytes: Vec<u8> = (0..8192).map(|i| ((i + seed_offset as usize) % 256) as u8).collect();
            let mut u = Unstructured::new(&bytes);
            if let Ok(program) = FuzzProgram::arbitrary(&mut u) {
                let html = serialize(
                    &program.font_faces,
                    &program.css_rules,
                    &program.dom,
                    &program.script,
                    &program.keyframes,
                    &program.at_rules,
                );
                // Basic validity checks
                assert!(html.starts_with("<!DOCTYPE html>"));
                assert!(html.contains("<html>"));
                assert!(html.contains("</html>"));
                // Should not contain raw null bytes
                assert!(!html.contains('\0'));
            }
        }
    }
}
