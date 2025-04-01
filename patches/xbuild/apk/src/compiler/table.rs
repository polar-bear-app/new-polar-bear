use crate::res::{Chunk, ResAttributeType, ResTableEntry, ResTableRef, ResTableValue, ResValue};
use anyhow::{Context, Result};
use std::io::Cursor;
use std::path::Path;

pub struct Ref<'a> {
    package: Option<&'a str>,
    ty: &'a str,
    name: &'a str,
}

impl<'a> Ref<'a> {
    pub fn attr(name: &'a str) -> Self {
        Self {
            package: Some("android"),
            ty: "attr",
            name,
        }
    }

    pub fn id(name: &'a str) -> Self {
        Self {
            package: Some("android"),
            ty: "id",
            name,
        }
    }

    pub fn parse(s: &'a str) -> Result<Self> {
        let s = s
            .strip_prefix('@')
            .with_context(|| format!("invalid reference {}: expected `@`", s))?;
        let (descr, name) = s
            .split_once('/')
            .with_context(|| format!("invalid reference {}: expected `/`", s))?;
        let (package, ty) = if let Some((package, ty)) = descr.split_once(':') {
            (Some(package), ty)
        } else {
            (None, descr)
        };
        Ok(Self { package, ty, name })
    }
}

struct Package<'a> {
    id: u8,
    types: &'a [String],
    keys: &'a [String],
    chunks: &'a [Chunk],
}

impl<'a> Package<'a> {
    fn new(id: u8, chunks: &'a [Chunk]) -> Result<Self> {
        let types = if let Chunk::StringPool(strings, _) = &chunks[0] {
            strings
        } else {
            anyhow::bail!("invalid package");
        };
        let keys = if let Chunk::StringPool(strings, _) = &chunks[1] {
            strings
        } else {
            anyhow::bail!("invalid package");
        };
        let chunks = &chunks[2..];
        Ok(Self {
            id,
            types,
            keys,
            chunks,
        })
    }

    fn lookup_type_id(&self, name: &str) -> Result<u8> {
        let id = self
            .types
            .iter()
            .position(|s| s.as_str() == name)
            .with_context(|| format!("failed to locate type id {}", name))?;
        Ok(id as u8 + 1)
    }

    fn lookup_key_id(&self, name: &str) -> Result<u32> {
        let id = self
            .keys
            .iter()
            .position(|s| s.as_str() == name)
            .with_context(|| format!("failed to locate key id {}", name))?;
        Ok(id as u32)
    }

    fn lookup_type(&self, id: u8) -> Result<Type<'a>> {
        for chunk in self.chunks {
            if let Chunk::TableType(header, _offsets, entries) = chunk {
                if header.id == id {
                    return Ok(Type {
                        package: self.id,
                        id,
                        entries,
                    });
                }
            }
        }
        anyhow::bail!("failed to locate type {}", id);
    }
}

struct Type<'a> {
    package: u8,
    id: u8,
    entries: &'a [Option<ResTableEntry>],
}

impl<'a> Type<'a> {
    pub fn lookup_entry_id(&self, key: u32) -> Result<u16> {
        let id = self
            .entries
            .iter()
            .position(|entry| {
                if let Some(entry) = entry {
                    entry.key == key
                } else {
                    false
                }
            })
            .with_context(|| format!("failed to lookup entry id {}", key))?;
        Ok(id as u16)
    }

    pub fn lookup_entry(&self, id: u16) -> Result<Entry<'a>> {
        let entry = self
            .entries
            .get(id as usize)
            .with_context(|| format!("failed to lookup entry {}", id))?
            .as_ref()
            .with_context(|| format!("failed to lookup entry {}", id))?;
        let id = ResTableRef::new(self.package, self.id, id);
        Ok(Entry { id, entry })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Entry<'a> {
    id: ResTableRef,
    entry: &'a ResTableEntry,
}

impl Entry<'_> {
    pub fn id(self) -> ResTableRef {
        self.id
    }

    pub fn attribute_type(self) -> Option<ResAttributeType> {
        if let ResTableValue::Complex(_, entries) = &self.entry.value {
            let data = entries[0].value.data;
            // TODO: android supports multiple types
            if data == 0b110 {
                return Some(ResAttributeType::Integer);
            }
            if data == 0b11 {
                return Some(ResAttributeType::String);
            }
            if data == 0b111110 {
                return Some(ResAttributeType::String);
            }
            if let Some(value) = ResAttributeType::from_u32(entries[0].value.data) {
                Some(value)
            } else {
                panic!("attribute_type: 0x{:x}", data);
            }
        } else {
            None
        }
    }

    pub fn lookup_value(&self, id: ResTableRef) -> Option<ResValue> {
        if let ResTableValue::Complex(_, entries) = &self.entry.value {
            for entry in &entries[1..] {
                if entry.name == u32::from(id) {
                    return Some(entry.value);
                }
            }
        }
        None
    }
}

#[derive(Default)]
pub struct Table {
    packages: Vec<Chunk>,
}

impl Table {
    pub fn import_apk(&mut self, apk: &Path) -> Result<()> {
        let resources = xcommon::extract_zip_file(apk, "resources.arsc")?;
        let chunk = Chunk::parse(&mut Cursor::new(resources))?;
        self.import_chunk(&chunk);
        Ok(())
    }

    pub fn import_chunk(&mut self, chunk: &Chunk) {
        if let Chunk::Table(_, packages) = chunk {
            self.packages.extend_from_slice(packages);
        }
    }

    fn lookup_package_id(&self, name: Option<&str>) -> Result<u8> {
        if let Some(name) = name {
            for package in &self.packages {
                if let Chunk::TablePackage(header, _) = package {
                    if header.name == name {
                        return Ok(header.id as u8);
                    }
                }
            }
            anyhow::bail!("failed to locate package {}", name);
        } else {
            Ok(127)
        }
    }

    fn lookup_package(&self, id: u8) -> Result<Package> {
        for package in &self.packages {
            if let Chunk::TablePackage(header, chunks) = package {
                if header.id == id as u32 {
                    return Package::new(id, chunks);
                }
            }
        }
        anyhow::bail!("failed to locate package {}", id);
    }

    pub fn entry_by_ref(&self, r: Ref) -> Result<Entry> {
        let id = self.lookup_package_id(r.package)?;
        let package = self.lookup_package(id)?;
        let id = package.lookup_type_id(r.ty)?;
        let ty = package.lookup_type(id)?;
        let key = package.lookup_key_id(r.name)?;
        let id = ty.lookup_entry_id(key)?;
        ty.lookup_entry(id)
    }

    /*pub fn entry(&self, r: ResTableRef) -> Result<Entry> {
        let package = self.lookup_package(r.package())?;
        let ty = package.lookup_type(r.ty())?;
        ty.lookup_entry(r.entry())
    }*/
}
