use super::*;

const SAMPLE: &str = r#"
// The Widget resource.
syntax = "proto3";

package demo.widget.v1;

import "demo/shared/v1/types.proto";
import "google/api/field_behavior.proto";

// A widget that does things.
message Widget {
  // Resource name of the widget.
  string name = 1 [(google.api.field_behavior) = IDENTIFIER];

  // How many things it does.
  int32 thing_count = 2 [(google.api.field_behavior) = OPTIONAL];

  // Tags applied to it.
  repeated string tags = 3;
}

// How a widget is shaped.
enum Shape {
  // Default value.
  SHAPE_UNSPECIFIED = 0;

  // Round.
  SHAPE_ROUND = 1;
}

// Manages widgets.
service Widgets {
  // Creates a widget.
  rpc CreateWidget(CreateWidgetRequest) returns (Widget);

  // Watches widgets change.
  rpc WatchWidgets(WatchWidgetsRequest) returns (stream Widget);
}
"#;

fn parsed() -> Module {
    let mut module = Module::default();
    parse_file(SAMPLE, &mut module);
    module
}

#[test]
fn reads_the_package() {
    assert_eq!(parsed().package, "demo.widget.v1");
}

#[test]
fn collects_imports() {
    let m = parsed();
    assert!(m.imports.contains(&"demo/shared/v1/types.proto".to_owned()));
    assert!(
        m.imports
            .contains(&"google/api/field_behavior.proto".to_owned())
    );
}

#[test]
fn reads_messages_and_their_comments() {
    let m = parsed();
    assert_eq!(m.messages.len(), 1);
    assert_eq!(m.messages[0].name, "Widget");
    assert_eq!(m.messages[0].comment, "A widget that does things.");
}

#[test]
fn reads_fields_with_numbers_and_behavior() {
    let m = parsed();
    let fields = &m.messages[0].fields;
    assert_eq!(fields.len(), 3);

    assert_eq!(fields[0].name, "name");
    assert_eq!(fields[0].number, 1);
    assert_eq!(fields[0].behavior, "IDENTIFIER");
    assert_eq!(fields[0].comment, "Resource name of the widget.");

    // The behaviour annotation is the difference between a field a caller must
    // set and one the server fills in, so it must survive parsing.
    assert_eq!(fields[1].behavior, "OPTIONAL");
    assert_eq!(fields[2].behavior, "", "no annotation means no behaviour");
}

#[test]
fn keeps_repeated_in_the_type() {
    let m = parsed();
    assert_eq!(m.messages[0].fields[2].type_name, "repeated string");
}

#[test]
fn reads_enums_and_values() {
    let m = parsed();
    assert_eq!(m.enums.len(), 1);
    assert_eq!(m.enums[0].name, "Shape");
    assert_eq!(m.enums[0].values.len(), 2);
    assert_eq!(m.enums[0].values[1].name, "SHAPE_ROUND");
    assert_eq!(m.enums[0].values[1].number, 1);
}

#[test]
fn reads_services_and_methods() {
    let m = parsed();
    assert_eq!(m.services.len(), 1);
    let rpcs = &m.services[0].rpcs;
    assert_eq!(rpcs.len(), 2);
    assert_eq!(rpcs[0].name, "CreateWidget");
    assert_eq!(rpcs[0].request, "CreateWidgetRequest");
    assert_eq!(rpcs[0].response, "Widget");
    assert!(!rpcs[0].server_stream);
}

#[test]
fn detects_a_streaming_response() {
    let m = parsed();
    let watch = &m.services[0].rpcs[1];
    assert!(watch.server_stream);
    assert_eq!(
        watch.response, "Widget",
        "the stream marker is not part of the type"
    );
}

#[test]
fn an_empty_file_parses_to_nothing() {
    let mut m = Module::default();
    parse_file("", &mut m);
    assert!(m.messages.is_empty() && m.services.is_empty() && m.enums.is_empty());
}

#[test]
fn a_blank_line_ends_a_comment_block() {
    // Otherwise a file-level comment would attach itself to whatever came next.
    let mut m = Module::default();
    parse_file("// Stray note.\n\nmessage Thing {\n}\n", &mut m);
    assert_eq!(m.messages[0].comment, "");
}
