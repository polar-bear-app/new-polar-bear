use crate::compiler::attributes::{StringPoolBuilder, Strings};
use crate::compiler::table::Table;
use crate::res::{
    Chunk, ResValue, ResValueType, ResXmlAttribute, ResXmlEndElement, ResXmlNamespace,
    ResXmlNodeHeader, ResXmlStartElement,
};
use anyhow::Result;
use roxmltree::{Document, Node, NodeType};
use std::collections::BTreeMap;

pub fn compile_xml(xml: &str, table: &Table) -> Result<Chunk> {
    let doc = Document::parse(xml)?;
    let root = doc.root_element();
    let mut builder = StringPoolBuilder::new(table);
    build_string_pool(root, &mut builder)?;
    let strings = builder.build();
    let mut chunks = vec![Chunk::Null, Chunk::Null];

    for ns in root.namespaces() {
        chunks.push(Chunk::XmlStartNamespace(
            ResXmlNodeHeader::default(),
            ResXmlNamespace {
                prefix: ns.name().map(|ns| strings.id(ns)).unwrap_or(-1),
                uri: strings.id(ns.uri()),
            },
        ));
    }
    compile_node(root, &strings, &mut chunks, table)?;
    for ns in root.namespaces() {
        chunks.push(Chunk::XmlEndNamespace(
            ResXmlNodeHeader::default(),
            ResXmlNamespace {
                prefix: ns.name().map(|ns| strings.id(ns)).unwrap_or(-1),
                uri: strings.id(ns.uri()),
            },
        ));
    }

    chunks[0] = Chunk::StringPool(strings.strings, vec![]);
    chunks[1] = Chunk::XmlResourceMap(strings.map);
    Ok(Chunk::Xml(chunks))
}

fn build_string_pool<'a>(node: Node<'a, 'a>, builder: &mut StringPoolBuilder<'a>) -> Result<()> {
    if node.node_type() != NodeType::Element {
        for node in node.children() {
            build_string_pool(node, builder)?;
        }
        return Ok(());
    }
    for ns in node.namespaces() {
        if let Some(prefix) = ns.name() {
            builder.add_string(prefix);
        }
        builder.add_string(ns.uri());
    }
    if let Some(ns) = node.tag_name().namespace() {
        builder.add_string(ns);
    }
    builder.add_string(node.tag_name().name());
    for attr in node.attributes() {
        builder.add_attribute(attr)?;
    }
    for node in node.children() {
        build_string_pool(node, builder)?;
    }
    Ok(())
}

fn compile_node(
    node: Node,
    strings: &Strings,
    chunks: &mut Vec<Chunk>,
    table: &Table,
) -> Result<()> {
    if node.node_type() != NodeType::Element {
        for node in node.children() {
            compile_node(node, strings, chunks, table)?;
        }
        return Ok(());
    }

    let mut id_index = 0;
    let mut class_index = 0;
    let mut style_index = 0;
    let mut attrs = BTreeMap::new();
    for (i, attr) in node.attributes().enumerate() {
        match attr.name() {
            "id" => id_index = i as u16 + 1,
            "class" => class_index = i as u16 + 1,
            "style" => style_index = i as u16 + 1,
            _ => {}
        }
        let value = if let Some("http://schemas.android.com/apk/res/android") = attr.namespace() {
            super::attributes::compile_attr(table, attr.name(), attr.value(), strings)?
        } else if attr.name() == "platformBuildVersionCode"
            || attr.name() == "platformBuildVersionName"
        {
            ResValue {
                size: 8,
                res0: 0,
                data_type: ResValueType::IntDec as u8,
                data: attr.value().parse()?,
            }
        } else {
            ResValue {
                size: 8,
                res0: 0,
                data_type: ResValueType::String as u8,
                data: strings.id(attr.value()) as u32,
            }
        };
        let raw_value = if value.data_type == ResValueType::String as u8 {
            value.data as i32
        } else {
            -1
        };
        let attr = ResXmlAttribute {
            namespace: attr.namespace().map(|ns| strings.id(ns)).unwrap_or(-1),
            name: strings.id(attr.name()),
            raw_value,
            typed_value: value,
        };
        attrs.insert(attr.name, attr);
    }
    let namespace = node
        .tag_name()
        .namespace()
        .map(|ns| strings.id(ns))
        .unwrap_or(-1);
    let name = strings.id(node.tag_name().name());
    chunks.push(Chunk::XmlStartElement(
        ResXmlNodeHeader::default(),
        ResXmlStartElement {
            namespace,
            name,
            attribute_start: 0x0014,
            attribute_size: 0x0014,
            attribute_count: attrs.len() as _,
            id_index,
            class_index,
            style_index,
        },
        attrs.into_values().collect(),
    ));
    /*let mut children = BTreeMap::new();
    for node in node.children() {
        children.insert(strings.id(node.tag_name().name()), node);
    }
    for (_, node) in children {
        compile_node(node, strings, chunks)?;
    }*/
    for node in node.children() {
        compile_node(node, strings, chunks, table)?;
    }
    chunks.push(Chunk::XmlEndElement(
        ResXmlNodeHeader::default(),
        ResXmlEndElement { namespace, name },
    ));
    Ok(())
}
