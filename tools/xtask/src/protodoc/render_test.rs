use super::*;
use crate::protodoc::model::{Field, Message, Rpc, Service};

fn module() -> Module {
    Module {
        name: "Widget".to_owned(),
        package: "demo.widget.v1".to_owned(),
        dir: "demo/widget".to_owned(),
        imports: Vec::new(),
        services: vec![Service {
            name: "Widgets".to_owned(),
            comment: "Manages widgets.".to_owned(),
            rpcs: vec![Rpc {
                name: "CreateWidget".to_owned(),
                comment: "Creates a widget.".to_owned(),
                request: "CreateWidgetRequest".to_owned(),
                response: "Widget".to_owned(),
                server_stream: false,
            }],
        }],
        messages: vec![Message {
            name: "Widget".to_owned(),
            comment: "A widget.".to_owned(),
            fields: vec![Field {
                name: "name".to_owned(),
                type_name: "string".to_owned(),
                number: 1,
                comment: "Resource name.".to_owned(),
                behavior: "IDENTIFIER".to_owned(),
            }],
        }],
        enums: Vec::new(),
    }
}

#[test]
fn carries_the_do_not_edit_banner() {
    // Without it, somebody edits a file that the next run silently overwrites.
    assert!(module_readme(&module()).starts_with("<!-- Generated"));
}

#[test]
fn documents_services_and_methods() {
    let out = module_readme(&module());
    assert!(out.contains("## Services"));
    assert!(out.contains("`CreateWidget`"));
    assert!(out.contains("Creates a widget."));
}

#[test]
fn documents_fields_with_behavior() {
    let out = module_readme(&module());
    assert!(out.contains("`IDENTIFIER`"));
    assert!(out.contains("Resource name."));
}

#[test]
fn marks_a_streaming_response() {
    let mut m = module();
    m.services[0].rpcs[0].server_stream = true;
    assert!(module_readme(&m).contains("stream `Widget`"));
}

#[test]
fn a_message_with_no_fields_says_so() {
    let mut m = module();
    m.messages[0].fields.clear();
    assert!(module_readme(&m).contains("_No fields._"));
}

#[test]
fn pipes_in_comments_are_escaped() {
    // An unescaped pipe splits the row into extra columns, which renders as a
    // broken table rather than failing.
    let mut m = module();
    m.messages[0].fields[0].comment = "either a | or b".to_owned();
    let out = module_readme(&m);
    assert!(out.contains("either a \\| or b"));
}

#[test]
fn an_empty_module_still_renders_a_header() {
    let m = Module {
        name: "Empty".to_owned(),
        package: "demo.empty.v1".to_owned(),
        ..Default::default()
    };
    let out = module_readme(&m);
    assert!(out.contains("# Empty"));
    assert!(!out.contains("## Services"));
}
