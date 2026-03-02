use std::fmt::Write;

use oxiom_ir::js::*;

/// Serialize a list of JS operations into a script block.
pub fn serialize_js_operations(ops: &[JsOperation]) -> String {
    let mut out = String::new();
    out.push_str("document.addEventListener('DOMContentLoaded', function() {\n");
    out.push_str("  var nodes = document.querySelectorAll('[id^=\"n\"]');\n");
    out.push_str("  function getNode(i) { return nodes[i % nodes.length]; }\n");
    out.push_str("  var sheet = document.styleSheets[0];\n");
    out.push_str("  try {\n");

    for op in ops {
        serialize_operation(op, &mut out, 2);
    }

    out.push_str("  } catch(e) { /* expected */ }\n");
    out.push_str("});\n");
    out
}

fn serialize_operation(op: &JsOperation, out: &mut String, indent: usize) {
    let pad: String = "  ".repeat(indent);

    match op {
        JsOperation::SetInlineStyle { target, property } => {
            let css_text = inline_style_text(property);
            write!(
                out,
                "{pad}try {{ getNode({}).style.cssText += '{}'; }} catch(e) {{}}\n",
                target.0, css_text
            )
            .unwrap();
        }

        JsOperation::RemoveInlineStyle {
            target,
            property_name,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).style.removeProperty('{}'); }} catch(e) {{}}\n",
                target.0,
                property_name.as_str()
            )
            .unwrap();
        }

        JsOperation::InsertStyleRule { rule_text, index } => {
            write!(
                out,
                "{pad}try {{ sheet.insertRule('{}', Math.min({}, sheet.cssRules.length)); }} catch(e) {{}}\n",
                escape_js(&rule_text.0),
                index
            )
            .unwrap();
        }

        JsOperation::DeleteStyleRule { index } => {
            write!(
                out,
                "{pad}try {{ sheet.deleteRule(Math.min({}, sheet.cssRules.length - 1)); }} catch(e) {{}}\n",
                index
            )
            .unwrap();
        }

        JsOperation::FontFaceLoad { family } => {
            write!(
                out,
                "{pad}try {{ document.fonts.load('16px \"{}\"'); }} catch(e) {{}}\n",
                escape_js(&family.0)
            )
            .unwrap();
        }

        JsOperation::FontFaceAdd { family, source } => {
            let src = match source {
                FontFaceSource::LocalArial => "local('Arial')",
                FontFaceSource::LocalTimesNewRoman => "local('Times New Roman')",
                FontFaceSource::EmptyUrl => "url('')",
                FontFaceSource::DataUrl => {
                    "url('data:font/woff2;base64,d09GMgABAAAAAA==')"
                }
            };
            write!(
                out,
                "{pad}try {{ var ff = new FontFace('{}', \"{}\"); document.fonts.add(ff); }} catch(e) {{}}\n",
                escape_js(&family.0),
                src
            )
            .unwrap();
        }

        JsOperation::FontFaceRemove { family } => {
            write!(
                out,
                "{pad}try {{ document.fonts.forEach(function(f) {{ if (f.family === '{}') document.fonts.delete(f); }}); }} catch(e) {{}}\n",
                escape_js(&family.0)
            )
            .unwrap();
        }

        JsOperation::AppendChild { parent, child } => {
            write!(
                out,
                "{pad}try {{ getNode({}).appendChild(getNode({})); }} catch(e) {{}}\n",
                parent.0, child.0
            )
            .unwrap();
        }

        JsOperation::RemoveChild { target } => {
            write!(
                out,
                "{pad}try {{ var t = getNode({}); if (t.parentNode) t.parentNode.removeChild(t); }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::CloneNode {
            source,
            deep,
            append_to,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).appendChild(getNode({}).cloneNode({})); }} catch(e) {{}}\n",
                append_to.0, source.0, deep
            )
            .unwrap();
        }

        JsOperation::GetOffsetWidth { target } => {
            write!(
                out,
                "{pad}try {{ void getNode({}).offsetWidth; }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::GetOffsetHeight { target } => {
            write!(
                out,
                "{pad}try {{ void getNode({}).offsetHeight; }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::GetBoundingClientRect { target } => {
            write!(
                out,
                "{pad}try {{ getNode({}).getBoundingClientRect(); }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::GetComputedStyle {
            target,
            property_name,
        } => {
            write!(
                out,
                "{pad}try {{ getComputedStyle(getNode({})).getPropertyValue('{}'); }} catch(e) {{}}\n",
                target.0,
                property_name.as_str()
            )
            .unwrap();
        }

        JsOperation::ForceGC => {
            write!(out, "{pad}try {{ if (typeof gc === 'function') gc(); }} catch(e) {{}}\n")
                .unwrap();
        }

        JsOperation::RequestAnimationFrame { operations } => {
            write!(out, "{pad}requestAnimationFrame(function() {{\n").unwrap();
            for inner_op in operations {
                serialize_operation(inner_op, out, indent + 1);
            }
            write!(out, "{pad}}});\n").unwrap();
        }

        JsOperation::SetTimeout {
            delay_ms,
            operations,
        } => {
            write!(out, "{pad}setTimeout(function() {{\n").unwrap();
            for inner_op in operations {
                serialize_operation(inner_op, out, indent + 1);
            }
            write!(out, "{pad}}}, {});\n", delay_ms).unwrap();
        }

        JsOperation::SetInnerHTML { target, html } => {
            write!(
                out,
                "{pad}try {{ getNode({}).innerHTML = '{}'; }} catch(e) {{}}\n",
                target.0,
                escape_js(&html.0)
            )
            .unwrap();
        }

        JsOperation::SetTextContent { target, text } => {
            write!(
                out,
                "{pad}try {{ getNode({}).textContent = '{}'; }} catch(e) {{}}\n",
                target.0,
                escape_js(&text.0)
            )
            .unwrap();
        }

        JsOperation::ToggleClass {
            target,
            class_name,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).classList.toggle('{}'); }} catch(e) {{}}\n",
                target.0,
                escape_js(&class_name.0)
            )
            .unwrap();
        }

        JsOperation::SetAttribute {
            target,
            attr_name,
            attr_value,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).setAttribute('{}', '{}'); }} catch(e) {{}}\n",
                target.0,
                attr_name.as_str(),
                escape_js(&attr_value.0)
            )
            .unwrap();
        }

        JsOperation::RemoveAttribute { target, attr_name } => {
            write!(
                out,
                "{pad}try {{ getNode({}).removeAttribute('{}'); }} catch(e) {{}}\n",
                target.0,
                attr_name.as_str()
            )
            .unwrap();
        }

        JsOperation::DispatchEvent {
            target,
            event_type,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).dispatchEvent(new Event('{}')); }} catch(e) {{}}\n",
                target.0,
                event_type.as_str()
            )
            .unwrap();
        }

        JsOperation::AdoptNode { target } => {
            write!(
                out,
                "{pad}try {{ document.adoptNode(getNode({})); }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::InsertAdjacentHTML {
            target,
            position,
            html,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).insertAdjacentHTML('{}', '{}'); }} catch(e) {{}}\n",
                target.0,
                position.as_str(),
                escape_js(&html.0)
            )
            .unwrap();
        }

        JsOperation::RangeDeleteContents {
            start_node,
            end_node,
        } => {
            write!(
                out,
                "{pad}try {{ var r = document.createRange(); r.setStartBefore(getNode({})); r.setEndAfter(getNode({})); r.deleteContents(); }} catch(e) {{}}\n",
                start_node.0, end_node.0
            )
            .unwrap();
        }

        // ========================================
        // Phase 3: New JS operations
        // ========================================

        JsOperation::AttachShadowRoot { target, mode } => {
            let mode_str = match mode {
                ShadowRootMode::Open => "open",
                ShadowRootMode::Closed => "closed",
            };
            write!(
                out,
                "{pad}try {{ getNode({}).attachShadow({{ mode: '{}' }}); }} catch(e) {{}}\n",
                target.0, mode_str
            )
            .unwrap();
        }

        JsOperation::ShadowRootSetInnerHTML { host, html } => {
            write!(
                out,
                "{pad}try {{ var sr = getNode({}).shadowRoot; if (sr) sr.innerHTML = '{}'; }} catch(e) {{}}\n",
                host.0,
                escape_js(&html.0)
            )
            .unwrap();
        }

        JsOperation::ObserveAndMutate {
            observe_target,
            mutate_target,
            mutation_op,
        } => {
            let mutation_js = match mutation_op {
                MutationOp::SetAttribute => {
                    format!("getNode({}).setAttribute('data-m','1')", mutate_target.0)
                }
                MutationOp::RemoveChild => {
                    format!("var t=getNode({});if(t.parentNode)t.parentNode.removeChild(t)", mutate_target.0)
                }
                MutationOp::AppendChild => {
                    format!("getNode({}).appendChild(document.createElement('div'))", mutate_target.0)
                }
                MutationOp::SetInnerHTML => {
                    format!("getNode({}).innerHTML='<b>mutated</b>'", mutate_target.0)
                }
                MutationOp::SetTextContent => {
                    format!("getNode({}).textContent='mutated'", mutate_target.0)
                }
            };
            write!(
                out,
                "{pad}try {{ var mo = new MutationObserver(function(muts) {{ try {{ {} }} catch(e) {{}} }}); mo.observe(getNode({}), {{ childList: true, attributes: true, subtree: true }}); {} }} catch(e) {{}}\n",
                mutation_js,
                observe_target.0,
                mutation_js
            )
            .unwrap();
        }

        JsOperation::InsertBefore {
            parent,
            new_child,
            ref_child,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).insertBefore(getNode({}), getNode({})); }} catch(e) {{}}\n",
                parent.0, new_child.0, ref_child.0
            )
            .unwrap();
        }

        JsOperation::ReplaceChild {
            parent,
            new_child,
            old_child,
        } => {
            write!(
                out,
                "{pad}try {{ getNode({}).replaceChild(getNode({}), getNode({})); }} catch(e) {{}}\n",
                parent.0, new_child.0, old_child.0
            )
            .unwrap();
        }

        JsOperation::ReplaceWith { target, html } => {
            write!(
                out,
                "{pad}try {{ var tmpl = document.createElement('template'); tmpl.innerHTML = '{}'; getNode({}).replaceWith(tmpl.content); }} catch(e) {{}}\n",
                escape_js(&html.0),
                target.0
            )
            .unwrap();
        }

        JsOperation::ExecCommand { command } => {
            if command.needs_value() {
                write!(
                    out,
                    "{pad}try {{ document.execCommand('{}', false, '{}'); }} catch(e) {{}}\n",
                    command.as_str(),
                    escape_js(command.default_value())
                )
                .unwrap();
            } else {
                write!(
                    out,
                    "{pad}try {{ document.execCommand('{}'); }} catch(e) {{}}\n",
                    command.as_str()
                )
                .unwrap();
            }
        }

        JsOperation::SelectAllContent => {
            write!(
                out,
                "{pad}try {{ var sel = window.getSelection(); sel.selectAllChildren(document.body); }} catch(e) {{}}\n"
            )
            .unwrap();
        }

        JsOperation::CollapseSelection { target } => {
            write!(
                out,
                "{pad}try {{ var sel = window.getSelection(); sel.collapse(getNode({}), 0); }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::CreateIframe { target, src_html } => {
            write!(
                out,
                "{pad}try {{ var ifr = document.createElement('iframe'); ifr.srcdoc = '{}'; getNode({}).appendChild(ifr); }} catch(e) {{}}\n",
                escape_js(&src_html.0),
                target.0
            )
            .unwrap();
        }

        JsOperation::RemoveIframe { target } => {
            write!(
                out,
                "{pad}try {{ var el = getNode({}); var ifrs = el.querySelectorAll('iframe'); ifrs.forEach(function(f) {{ f.remove(); }}); }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::ResizeObserverObserve {
            target,
            callback_ops,
        } => {
            write!(out, "{pad}try {{ var ro = new ResizeObserver(function() {{\n").unwrap();
            for inner_op in callback_ops {
                serialize_operation(inner_op, out, indent + 1);
            }
            write!(
                out,
                "{pad}}}); ro.observe(getNode({})); }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::IntersectionObserverObserve {
            target,
            callback_ops,
        } => {
            write!(
                out,
                "{pad}try {{ var io = new IntersectionObserver(function() {{\n"
            )
            .unwrap();
            for inner_op in callback_ops {
                serialize_operation(inner_op, out, indent + 1);
            }
            write!(
                out,
                "{pad}}}); io.observe(getNode({})); }} catch(e) {{}}\n",
                target.0
            )
            .unwrap();
        }

        JsOperation::QueueMicrotask { operations } => {
            write!(out, "{pad}queueMicrotask(function() {{\n").unwrap();
            for inner_op in operations {
                serialize_operation(inner_op, out, indent + 1);
            }
            write!(out, "{pad}}});\n").unwrap();
        }
    }
}

fn inline_style_text(property: &oxiom_ir::css::CssProperty) -> String {
    let mut out = String::new();
    oxiom_serializer_css::serialize_property(property, &mut out);
    escape_js(&out)
}

fn escape_js(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c => out.push(c),
        }
    }
    out
}

/// Module alias for CSS serialization used by JS serializer.
mod oxiom_serializer_css {
    pub use crate::css::serialize_property;
}
