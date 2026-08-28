//! Building the crate root from the discovered modules.
//!
//! Two layers, and the split is forced rather than chosen. Both compilers emit
//! code that addresses its siblings absolutely — `crate::message_capnp` — so
//! every module must exist at the crate root or the generated code stops
//! compiling. Nesting them properly would mean editing generated files.
//!
//! So the roots stay flat and hidden, and a facade re-exports each one under
//! the package it came from. The files on disk mirror the protos, the public
//! API mirrors the protos, and not one generated line is touched.

use crate::gen_buffers::tree::{GeneratedModule, Modules, ProtoPackage};
use std::collections::BTreeMap;

/// One node of the facade: modules declared here, and packages beneath.
#[derive(Default)]
struct Node<'a> {
    /// Modules that belong directly at this level.
    modules: Vec<&'a str>,
    /// Nested packages, ordered so output is stable across runs.
    children: BTreeMap<&'a str, Node<'a>>,
}

/// Insert one module at its package path.
fn insert<'a>(node: &mut Node<'a>, package: &'a [String], name: &'a str) {
    match package.split_first() {
        None => node.modules.push(name),
        Some((head, rest)) => {
            insert(node.children.entry(head.as_str()).or_default(), rest, name)
        }
    }
}

/// Render one node and everything under it.
fn render(node: &Node<'_>, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let mut s = String::new();
    if !node.modules.is_empty() {
        s.push_str(&format!("{pad}pub use crate::{{\n"));
        for m in &node.modules {
            s.push_str(&format!("{pad}    {m},\n"));
        }
        s.push_str(&format!("{pad}}};\n"));
    }
    for (name, child) in &node.children {
        s.push_str(&format!("{pad}/// `{name}` schemas.\n"));
        s.push_str(&format!("{pad}pub mod {name} {{\n"));
        s.push_str(&render(child, indent + 4));
        s.push_str(&format!("{pad}}}\n"));
    }
    s
}

/// Hidden root declarations plus the nested facade, for both targets.
pub fn lib_rs(preamble: &str, m: &Modules) -> String {
    let targets: [(&str, &str, &str, &Vec<GeneratedModule>); 2] = [
        (
            "capnp",
            "capnp",
            "Cap'n Proto views. Validated reads, no `unsafe`, on by default.",
            &m.capnp,
        ),
        (
            "flatbuffers",
            "flatbuffers",
            "FlatBuffers views. Zero-copy reads, `unsafe` accessors, opt-in.",
            &m.flatbuffers,
        ),
    ];

    let mut s = String::from(preamble);
    s.push_str(
        "\n// Roots the generators require at the crate root. Hidden: the grouped\n\
         // modules below are the surface a caller is meant to find.\n\n",
    );
    for (dir, feature, _, mods) in &targets {
        for gm in mods.iter() {
            s.push_str(&format!(
                "#[cfg(feature = \"{feature}\")]\n\
                 #[path = \"generated/{dir}/{}\"]\n\
                 #[doc(hidden)]\npub mod {};\n",
                gm.path, gm.name
            ));
        }
        s.push('\n');
    }

    s.push_str(&protobuf(&m.protobuf));
    s.push_str("// The documented surface: one module per format, nested by package.\n\n");
    for (_, feature, doc, mods) in &targets {
        let mut root = Node::default();
        for gm in mods.iter() {
            insert(&mut root, &gm.package, &gm.name);
        }
        s.push_str(&format!(
            "/// {doc}\n#[cfg(feature = \"{feature}\")]\npub mod {feature} {{\n"
        ));
        s.push_str(&render(&root, 4));
        s.push_str("}\n\n");
    }
    s
}

/// Render the protobuf tree, which nests directly with no facade.
fn protobuf(packages: &[ProtoPackage]) -> String {
    if packages.is_empty() {
        return String::new();
    }
    let mut s = String::from(
        "/// Protobuf views and the gRPC service stubs, the wire protocol the
         /// server speaks. Nested directly: `prost` addresses siblings
         /// relatively, so unlike the flat formats these need no root layer.
         #[cfg(feature = \"protobuf\")]
pub mod protobuf {
",
    );
    for p in packages {
        let indent = "    ".repeat(p.package.len());
        for (depth, seg) in p.package.iter().enumerate() {
            let pad = "    ".repeat(depth + 1);
            s.push_str(&format!("{pad}/// `{seg}`.\n{pad}pub mod {seg} {{\n"));
        }
        for inc in &p.includes {
            s.push_str(&format!("{indent}    include!(\"{inc}\");\n"));
        }
        for depth in (0..p.package.len()).rev() {
            s.push_str(&format!("{}}}\n", "    ".repeat(depth + 1)));
        }
    }
    s.push_str(
        "    /// Serialized `FileDescriptorSet` for every service here.
         ///
         /// Served over gRPC reflection so generic clients can introspect the
         /// API without being shipped the protos first.
         pub const FILE_DESCRIPTOR_SET: &[u8] =
         \x20       include_bytes!(\"generated/protobuf/descriptor.bin\");
}

",
    );
    s
}
