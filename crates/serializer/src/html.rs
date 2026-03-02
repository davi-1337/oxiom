use std::fmt::Write;

use oxiom_ir::html::*;

/// Serialize a DomTree into HTML body content, assigning ids n0, n1, ...
/// Phase 2: raised node cap from 256 to 512.
pub fn serialize_dom_tree(tree: &DomTree) -> (String, usize) {
    let mut out = String::new();
    let mut counter = 0usize;
    for child in &tree.root_children {
        serialize_dom_node(child, &mut out, &mut counter);
    }
    (out, counter)
}

fn serialize_dom_node(node: &DomNode, out: &mut String, counter: &mut usize) {
    let id = *counter;
    *counter += 1;

    let tag = node.element.tag_name();

    if node.element.is_void() {
        write!(out, "<{} id=\"n{}\">", tag, id).unwrap();
        return;
    }

    write!(out, "<{} id=\"n{}\">", tag, id).unwrap();

    if let Some(text) = &node.text_content {
        // Escape HTML entities in text content
        for ch in text.0.chars() {
            match ch {
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '&' => out.push_str("&amp;"),
                '"' => out.push_str("&quot;"),
                '\0' => {} // skip null bytes
                c => out.push(c),
            }
        }
    }

    // Limit recursion depth — raised from 256 to 512
    if *counter < 512 {
        for child in &node.children {
            serialize_dom_node(child, out, counter);
        }
    }

    write!(out, "</{}>", tag).unwrap();
}
