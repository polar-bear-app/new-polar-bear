use crate::compiler::table::{Ref, Table};
use crate::res::{ResAttributeType, ResValue, ResValueType};
use anyhow::{Context, Result};
use roxmltree::Attribute;
use std::collections::{BTreeMap, BTreeSet};

pub fn compile_attr(table: &Table, name: &str, value: &str, strings: &Strings) -> Result<ResValue> {
    let entry = table.entry_by_ref(Ref::attr(name))?;
    let attr_type = entry.attribute_type().unwrap();
    let (data, data_type) = match attr_type {
        ResAttributeType::Reference => {
            let id = table.entry_by_ref(Ref::parse(value)?)?.id();
            (u32::from(id), ResValueType::Reference)
        }
        ResAttributeType::String => (strings.id(value) as u32, ResValueType::String),
        ResAttributeType::Integer => (value.parse()?, ResValueType::IntDec),
        ResAttributeType::Boolean => match value {
            "true" => (0xffff_ffff, ResValueType::IntBoolean),
            "false" => (0x0000_0000, ResValueType::IntBoolean),
            _ => anyhow::bail!("expected boolean"),
        },
        ResAttributeType::Enum => {
            let id = table.entry_by_ref(Ref::id(value))?.id();
            let value = entry.lookup_value(id).unwrap();
            (value.data, ResValueType::from_u8(value.data_type).unwrap())
        }
        ResAttributeType::Flags => {
            let mut data = 0;
            let mut data_type = ResValueType::Null;
            for flag in value.split('|') {
                let id = table.entry_by_ref(Ref::id(flag))?.id();
                let value = entry.lookup_value(id).unwrap();
                data |= value.data;
                data_type = ResValueType::from_u8(value.data_type).unwrap();
            }
            (data, data_type)
        }
        _ => anyhow::bail!("unsupported attribute type"),
    };
    Ok(ResValue {
        size: 8,
        res0: 0,
        data_type: data_type as u8,
        data,
    })
}

pub struct StringPoolBuilder<'a> {
    table: &'a Table,
    attributes: BTreeMap<u32, &'a str>,
    strings: BTreeSet<&'a str>,
}

impl<'a> StringPoolBuilder<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self {
            table,
            attributes: Default::default(),
            strings: Default::default(),
        }
    }

    pub fn add_attribute(&mut self, attr: Attribute<'a, 'a>) -> Result<()> {
        if let Some(ns) = attr.namespace() {
            if ns == "http://schemas.android.com/apk/res/android" {
                let entry = self.table.entry_by_ref(Ref::attr(attr.name()))?;
                self.attributes.insert(entry.id().into(), attr.name());
                if entry.attribute_type() == Some(ResAttributeType::String) {
                    self.strings.insert(attr.value());
                }
                return Ok(());
            }
        }
        if attr.name() == "platformBuildVersionCode" || attr.name() == "platformBuildVersionName" {
            self.strings.insert(attr.name());
        } else {
            self.strings.insert(attr.name());
            self.strings.insert(attr.value());
        }
        Ok(())
    }

    pub fn add_string(&mut self, s: &'a str) {
        self.strings.insert(s);
    }

    pub fn build(self) -> Strings {
        let mut strings = Vec::with_capacity(self.attributes.len() + self.strings.len());
        let mut map = Vec::with_capacity(self.attributes.len());
        for (id, name) in self.attributes {
            strings.push(name.to_string());
            map.push(id);
        }
        for string in self.strings {
            strings.push(string.to_string());
        }
        Strings { strings, map }
    }
}

pub struct Strings {
    pub strings: Vec<String>,
    pub map: Vec<u32>,
}

impl Strings {
    pub fn id(&self, s2: &str) -> i32 {
        self.strings
            .iter()
            .position(|s| s == s2)
            .with_context(|| format!("all strings added to the string pool: {}", s2))
            .unwrap() as i32
    }
}
