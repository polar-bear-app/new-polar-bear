use anyhow::{ensure, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HierarchicalSchema {
    unique_name: String,
    name: String,
    scopes: Vec<ResourceMapEntry>,
    items: Vec<ResourceMapEntry>,
}

impl HierarchicalSchema {
    pub const IDENTIFIER: &'static [u8; 16] = b"[mrm_hschemaex] ";

    pub fn read<R: Read + Seek>(r: &mut R) -> Result<Self> {
        ensure!(r.read_u16::<LE>()? == 1);
        let unique_name_length = r.read_u16::<LE>()? as usize;
        let name_length = r.read_u16::<LE>()? as usize;
        ensure!(r.read_u16::<LE>()? == 0);
        let mut hnames = [0; 16];
        r.read_exact(&mut hnames)?;
        ensure!(hnames == *b"[def_hnamesx]  \0");
        let _major_version = r.read_u16::<LE>()?;
        let _minor_version = r.read_u16::<LE>()?;
        ensure!(r.read_u32::<LE>()? == 0);
        let _checksum = r.read_u32::<LE>()?;
        let num_scopes = r.read_u32::<LE>()? as usize;
        let num_items = r.read_u32::<LE>()? as usize;
        let mut unique_name = String::with_capacity(unique_name_length);
        loop {
            let c = r.read_u16::<LE>()?;
            if c == 0 {
                break;
            }
            unique_name.push(char::from_u32(c as u32).unwrap());
        }
        ensure!(unique_name.len() + 1 == unique_name_length);
        let mut name = String::with_capacity(name_length);
        loop {
            let c = r.read_u16::<LE>()?;
            if c == 0 {
                break;
            }
            name.push(char::from_u32(c as u32).unwrap());
        }
        ensure!(name.len() + 1 == name_length);
        ensure!(r.read_u16::<LE>()? == 0);
        let _max_full_path_length = r.read_u16::<LE>();
        ensure!(r.read_u16::<LE>()? == 0);
        ensure!(r.read_u32::<LE>()? as usize == num_scopes + num_items);
        ensure!(r.read_u32::<LE>()? as usize == num_scopes);
        ensure!(r.read_u32::<LE>()? as usize == num_items);
        let unicode_data_length = r.read_u32::<LE>()? as u64;
        r.read_u32::<LE>()?;
        r.read_u32::<LE>()?;
        let mut scope_and_item_infos = Vec::with_capacity(num_scopes + num_items);
        for _ in 0..(num_scopes + num_items) {
            scope_and_item_infos.push(ScopeAndItemInfo::read(r)?);
        }
        let mut scope_ex_infos = Vec::with_capacity(num_scopes);
        for _ in 0..num_scopes {
            scope_ex_infos.push(ScopeExInfo::read(r)?);
        }
        let mut item_index_property_to_index = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            item_index_property_to_index.push(r.read_u16::<LE>()?);
        }
        let unicode_data_offset = r.stream_position()?;
        let ascii_data_offset = unicode_data_offset + unicode_data_length * 2;
        let mut scopes = vec![ResourceMapEntry::default(); num_scopes];
        let mut items = vec![ResourceMapEntry::default(); num_items];
        for info in &scope_and_item_infos {
            let pos = if info.name_in_ascii {
                ascii_data_offset + info.name_offset
            } else {
                unicode_data_offset + info.name_offset * 2
            };
            r.seek(SeekFrom::Start(pos))?;
            let mut name = String::with_capacity(info.full_path_length);
            if info.full_path_length != 0 {
                if info.name_in_ascii {
                    loop {
                        let c = r.read_u8()?;
                        if c == 0 {
                            break;
                        }
                        name.push(char::from_u32(c as u32).unwrap());
                    }
                } else {
                    loop {
                        let c = r.read_u16::<LE>()?;
                        if c == 0 {
                            break;
                        }
                        name.push(char::from_u32(c as u32).unwrap());
                    }
                }
            }
            let parent = if info.parent != 0xffff {
                Some(info.parent)
            } else {
                None
            };
            let entry = ResourceMapEntry { parent, name };
            if info.is_scope {
                scopes[info.index] = entry;
            } else {
                items[info.index] = entry;
            }
        }
        Ok(Self {
            unique_name,
            name,
            scopes,
            items,
        })
    }

    pub fn write<W: Write + Seek>(&self, w: &mut W) -> Result<()> {
        w.write_u16::<LE>(1)?;
        w.write_u16::<LE>(self.unique_name.len() as u16 + 1)?;
        w.write_u16::<LE>(self.name.len() as u16 + 1)?;
        w.write_u16::<LE>(0)?;
        w.write_all(b"[def_hnamesx]  \0")?;
        w.write_u16::<LE>(1)?;
        w.write_u16::<LE>(0)?;
        w.write_u32::<LE>(0)?;
        // TODO: checksum
        w.write_u32::<LE>(0)?;
        w.write_u32::<LE>(self.scopes.len() as u32)?;
        w.write_u32::<LE>(self.items.len() as u32)?;
        for c in self.unique_name.chars() {
            w.write_u16::<LE>(c as u16)?;
        }
        w.write_u16::<LE>(0)?;
        for c in self.name.chars() {
            w.write_u16::<LE>(c as u16)?;
        }
        w.write_u16::<LE>(0)?;
        w.write_u16::<LE>(0)?;
        // TODO: max full path length
        w.write_u16::<LE>(256)?;
        w.write_u16::<LE>(0)?;
        w.write_u32::<LE>((self.scopes.len() + self.items.len()) as u32)?;
        w.write_u32::<LE>(self.scopes.len() as u32)?;
        w.write_u32::<LE>(self.items.len() as u32)?;
        // TODO: unicode_data_length
        w.write_u32::<LE>(0)?;
        // TODO: what's this for
        w.write_u32::<LE>(0)?;
        // TODO: what's this for
        w.write_u32::<LE>(0)?;

        let mut scope_and_item_infos = Vec::with_capacity(self.scopes.len() + self.items.len());
        let mut scope_ex_infos = Vec::with_capacity(self.scopes.len());
        let mut item_index_property_to_index = Vec::with_capacity(self.items.len());
        let mut unicode_strings = vec![];
        for (i, scope) in self.scopes.iter().enumerate() {
            let scope_info = ScopeAndItemInfo {
                parent: scope.parent.unwrap_or(0xffff),
                full_path_length: scope.name.len(),
                is_scope: true,
                name_in_ascii: false,
                name_offset: unicode_strings.len() as u64 / 2,
                index: i,
            };
            scope_and_item_infos.push(scope_info);
            scope_ex_infos.push(ScopeExInfo {
                scope_index: i as u16,
                // TODO: child_count
                child_count: 0,
                // TODO: first_child_index
                first_child_index: 0,
            });
            for c in scope.name.chars() {
                unicode_strings.write_u16::<LE>(c as u16)?;
            }
            unicode_strings.write_u16::<LE>(0)?;
        }
        for (i, item) in self.items.iter().enumerate() {
            let item_info = ScopeAndItemInfo {
                parent: item.parent.unwrap_or(0xffff),
                full_path_length: item.name.len(),
                is_scope: false,
                name_in_ascii: false,
                name_offset: unicode_strings.len() as u64 / 2,
                index: i,
            };
            scope_and_item_infos.push(item_info);
            // TODO: item_index_property_to_index
            item_index_property_to_index.push(0);
            for c in item.name.chars() {
                unicode_strings.write_u16::<LE>(c as u16)?;
            }
            unicode_strings.write_u16::<LE>(0)?;
        }
        for scope_and_item_info in scope_and_item_infos {
            scope_and_item_info.write(w)?;
        }
        for scope_ex_info in scope_ex_infos {
            scope_ex_info.write(w)?;
        }
        for index in item_index_property_to_index {
            w.write_u16::<LE>(index)?;
        }
        w.write_all(&unicode_strings)?;
        Ok(())
    }
}

struct ScopeAndItemInfo {
    parent: usize,
    full_path_length: usize,
    is_scope: bool,
    name_in_ascii: bool,
    name_offset: u64,
    index: usize,
}

impl ScopeAndItemInfo {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let parent = r.read_u16::<LE>()? as usize;
        let full_path_length = r.read_u16::<LE>()? as usize;
        let _uppercase_first_char = r.read_u16::<LE>()?;
        let _name_length_2 = r.read_u8()?;
        let flags = r.read_u8()?;
        let name_offset = r.read_u16::<LE>()? as u64 | (flags as u64 & 0xf << 16);
        let index = r.read_u16::<LE>()? as usize;
        let is_scope = flags & 0x10 > 0;
        let name_in_ascii = flags & 0x20 > 0;
        Ok(Self {
            parent,
            full_path_length,
            name_offset,
            index,
            is_scope,
            name_in_ascii,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u16::<LE>(self.parent as u16)?;
        w.write_u16::<LE>(self.full_path_length as u16)?;
        w.write_u16::<LE>(0)?;
        w.write_u8(0)?;
        let mut flags = (self.name_offset >> 16) as u8 & 0xf;
        if self.is_scope {
            flags |= 0x10;
        }
        if self.name_in_ascii {
            flags |= 0x20;
        }
        w.write_u8(flags)?;
        w.write_u16::<LE>(self.name_offset as u16)?;
        w.write_u16::<LE>(self.index as u16)?;
        Ok(())
    }
}

struct ScopeExInfo {
    scope_index: u16,
    child_count: u16,
    first_child_index: u16,
}

impl ScopeExInfo {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let scope_index = r.read_u16::<LE>()?;
        let child_count = r.read_u16::<LE>()?;
        let first_child_index = r.read_u16::<LE>()?;
        ensure!(r.read_u16::<LE>()? == 0);
        Ok(Self {
            scope_index,
            child_count,
            first_child_index,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u16::<LE>(self.scope_index)?;
        w.write_u16::<LE>(self.child_count)?;
        w.write_u16::<LE>(self.first_child_index)?;
        w.write_u16::<LE>(0)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResourceMapEntry {
    pub parent: Option<usize>,
    pub name: String,
}
