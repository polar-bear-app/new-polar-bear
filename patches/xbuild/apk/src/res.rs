use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum ChunkType {
    Null = 0x0000,
    StringPool = 0x0001,
    Table = 0x0002,
    Xml = 0x0003,
    XmlStartNamespace = 0x0100,
    XmlEndNamespace = 0x0101,
    XmlStartElement = 0x0102,
    XmlEndElement = 0x0103,
    //XmlCdata = 0x0104,
    //XmlLastChunk = 0x017f,
    XmlResourceMap = 0x0180,
    TablePackage = 0x0200,
    TableType = 0x0201,
    TableTypeSpec = 0x0202,
    Unknown = 0x0206,
}

impl ChunkType {
    pub fn from_u16(ty: u16) -> Option<Self> {
        Some(match ty {
            ty if ty == ChunkType::Null as u16 => ChunkType::Null,
            ty if ty == ChunkType::StringPool as u16 => ChunkType::StringPool,
            ty if ty == ChunkType::Table as u16 => ChunkType::Table,
            ty if ty == ChunkType::Xml as u16 => ChunkType::Xml,
            ty if ty == ChunkType::XmlStartNamespace as u16 => ChunkType::XmlStartNamespace,
            ty if ty == ChunkType::XmlEndNamespace as u16 => ChunkType::XmlEndNamespace,
            ty if ty == ChunkType::XmlStartElement as u16 => ChunkType::XmlStartElement,
            ty if ty == ChunkType::XmlEndElement as u16 => ChunkType::XmlEndElement,
            //ty if ty == ChunkType::XmlCdata as u16 => ChunkType::XmlCdata,
            //ty if ty == ChunkType::XmlLastChunk as u16 => ChunkType::XmlLastChunk,
            ty if ty == ChunkType::XmlResourceMap as u16 => ChunkType::XmlResourceMap,
            ty if ty == ChunkType::TablePackage as u16 => ChunkType::TablePackage,
            ty if ty == ChunkType::TableType as u16 => ChunkType::TableType,
            ty if ty == ChunkType::TableTypeSpec as u16 => ChunkType::TableTypeSpec,
            ty if ty == ChunkType::Unknown as u16 => ChunkType::Unknown,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ResChunkHeader {
    /// Type identifier for this chunk. The meaning of this value depends
    /// on the containing chunk.
    pub ty: u16,
    /// Size of the chunk header (in bytes). Adding this value to the address
    /// of the chunk allows you to find its associated data (if any).
    pub header_size: u16,
    /// Total size of this chunk (in bytes). This is the header_size plus the
    /// size of any data associated with the chunk. Adding this value to the
    /// chunk allows you to completely skip its contents (including any child
    /// chunks). If this value is the same as header_size, there is no data
    /// associated with the chunk.
    pub size: u32,
}

impl ResChunkHeader {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let ty = r.read_u16::<LittleEndian>()?;
        let header_size = r.read_u16::<LittleEndian>()?;
        let size = r.read_u32::<LittleEndian>()?;
        Ok(Self {
            ty,
            header_size,
            size,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u16::<LittleEndian>(self.ty)?;
        w.write_u16::<LittleEndian>(self.header_size)?;
        w.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ResStringPoolHeader {
    pub string_count: u32,
    pub style_count: u32,
    pub flags: u32,
    pub strings_start: u32,
    pub styles_start: u32,
}

impl ResStringPoolHeader {
    pub const SORTED_FLAG: u32 = 1 << 0;
    pub const UTF8_FLAG: u32 = 1 << 8;

    pub fn read(r: &mut impl Read) -> Result<Self> {
        let string_count = r.read_u32::<LittleEndian>()?;
        let style_count = r.read_u32::<LittleEndian>()?;
        let flags = r.read_u32::<LittleEndian>()?;
        let strings_start = r.read_u32::<LittleEndian>()?;
        let styles_start = r.read_u32::<LittleEndian>()?;
        Ok(Self {
            string_count,
            style_count,
            flags,
            strings_start,
            styles_start,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.string_count)?;
        w.write_u32::<LittleEndian>(self.style_count)?;
        w.write_u32::<LittleEndian>(self.flags)?;
        w.write_u32::<LittleEndian>(self.strings_start)?;
        w.write_u32::<LittleEndian>(self.styles_start)?;
        Ok(())
    }

    pub fn is_utf8(&self) -> bool {
        self.flags & Self::UTF8_FLAG > 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResTableHeader {
    pub package_count: u32,
}

impl ResTableHeader {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let package_count = r.read_u32::<LittleEndian>()?;
        Ok(Self { package_count })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.package_count)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResXmlNodeHeader {
    pub line_number: u32,
    pub comment: i32,
}

impl ResXmlNodeHeader {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let _line_number = r.read_u32::<LittleEndian>()?;
        let _comment = r.read_i32::<LittleEndian>()?;
        Ok(Self {
            line_number: 1,
            comment: -1,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.line_number)?;
        w.write_i32::<LittleEndian>(self.comment)?;
        Ok(())
    }
}

impl Default for ResXmlNodeHeader {
    fn default() -> Self {
        Self {
            line_number: 1,
            comment: -1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResXmlNamespace {
    pub prefix: i32,
    pub uri: i32,
}

impl ResXmlNamespace {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let prefix = r.read_i32::<LittleEndian>()?;
        let uri = r.read_i32::<LittleEndian>()?;
        Ok(Self { prefix, uri })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.prefix)?;
        w.write_i32::<LittleEndian>(self.uri)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResXmlStartElement {
    /// String of the full namespace of this element.
    pub namespace: i32,
    /// String name of this node if it is an ELEMENT; the raw
    /// character data if this is a CDATA node.
    pub name: i32,
    /// Byte offset from the start of this structure to where
    /// the attributes start.
    pub attribute_start: u16,
    /// Size of the attribute structures that follow.
    pub attribute_size: u16,
    /// Number of attributes associated with an ELEMENT. These are
    /// available as an array of ResXmlAttribute structures
    /// immediately following this node.
    pub attribute_count: u16,
    /// Index (1-based) of the "id" attribute. 0 if none.
    pub id_index: u16,
    /// Index (1-based) of the "class" attribute. 0 if none.
    pub class_index: u16,
    /// Index (1-based) of the "style" attribute. 0 if none.
    pub style_index: u16,
}

impl Default for ResXmlStartElement {
    fn default() -> Self {
        Self {
            namespace: -1,
            name: -1,
            attribute_start: 0x0014,
            attribute_size: 0x0014,
            attribute_count: 0,
            id_index: 0,
            class_index: 0,
            style_index: 0,
        }
    }
}

impl ResXmlStartElement {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let namespace = r.read_i32::<LittleEndian>()?;
        let name = r.read_i32::<LittleEndian>()?;
        let attribute_start = r.read_u16::<LittleEndian>()?;
        let attribute_size = r.read_u16::<LittleEndian>()?;
        let attribute_count = r.read_u16::<LittleEndian>()?;
        let id_index = r.read_u16::<LittleEndian>()?;
        let class_index = r.read_u16::<LittleEndian>()?;
        let style_index = r.read_u16::<LittleEndian>()?;
        Ok(Self {
            namespace,
            name,
            attribute_start,
            attribute_size,
            attribute_count,
            id_index,
            class_index,
            style_index,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.namespace)?;
        w.write_i32::<LittleEndian>(self.name)?;
        w.write_u16::<LittleEndian>(self.attribute_start)?;
        w.write_u16::<LittleEndian>(self.attribute_size)?;
        w.write_u16::<LittleEndian>(self.attribute_count)?;
        w.write_u16::<LittleEndian>(self.id_index)?;
        w.write_u16::<LittleEndian>(self.class_index)?;
        w.write_u16::<LittleEndian>(self.style_index)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResXmlAttribute {
    pub namespace: i32,
    pub name: i32,
    pub raw_value: i32,
    pub typed_value: ResValue,
}

impl ResXmlAttribute {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let namespace = r.read_i32::<LittleEndian>()?;
        let name = r.read_i32::<LittleEndian>()?;
        let raw_value = r.read_i32::<LittleEndian>()?;
        let typed_value = ResValue::read(r)?;
        Ok(Self {
            namespace,
            name,
            raw_value,
            typed_value,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.namespace)?;
        w.write_i32::<LittleEndian>(self.name)?;
        w.write_i32::<LittleEndian>(self.raw_value)?;
        self.typed_value.write(w)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResXmlEndElement {
    pub namespace: i32,
    pub name: i32,
}

impl ResXmlEndElement {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let namespace = r.read_i32::<LittleEndian>()?;
        let name = r.read_i32::<LittleEndian>()?;
        Ok(Self { namespace, name })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.namespace)?;
        w.write_i32::<LittleEndian>(self.name)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResTableRef(u32);

impl ResTableRef {
    pub fn new(package: u8, ty: u8, entry: u16) -> Self {
        let package = (package as u32) << 24;
        let ty = (ty as u32) << 16;
        let entry = entry as u32;
        Self(package | ty | entry)
    }

    pub fn package(self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn ty(self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn entry(self) -> u16 {
        self.0 as u16
    }
}

impl From<u32> for ResTableRef {
    fn from(r: u32) -> Self {
        Self(r)
    }
}

impl From<ResTableRef> for u32 {
    fn from(r: ResTableRef) -> u32 {
        r.0
    }
}

impl std::fmt::Display for ResTableRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResTablePackageHeader {
    /// If this is a base package, its ID. Package IDs start
    /// at 1 (corresponding to the value of the package bits in a
    /// resource identifier). 0 means this is not a base package.
    pub id: u32,
    /// Actual name of this package, \0-terminated.
    pub name: String,
    /// Offset to a ResStringPoolHeader defining the resource
    /// type symbol table. If zero, this package is inheriting
    /// from another base package (overriding specific values in it).
    pub type_strings: u32,
    /// Last index into type_strings that is for public use by others.
    pub last_public_type: u32,
    /// Offset to a ResStringPoolHeader defining the resource key
    /// symbol table. If zero, this package is inheriting from another
    /// base package (overriding specific values in it).
    pub key_strings: u32,
    /// Last index into key_strings that is for public use by others.
    pub last_public_key: u32,
    pub type_id_offset: u32,
}

impl ResTablePackageHeader {
    pub fn read<R: Read + Seek>(r: &mut R) -> Result<Self> {
        let id = r.read_u32::<LittleEndian>()?;
        let mut name = [0; 128];
        let mut name_len = 0xff;
        for (i, item) in name.iter_mut().enumerate() {
            let c = r.read_u16::<LittleEndian>()?;
            if name_len < 128 {
                continue;
            }
            if c == 0 {
                name_len = i;
            } else {
                *item = c;
            }
        }
        let name = String::from_utf16(&name[..name_len])?;
        let type_strings = r.read_u32::<LittleEndian>()?;
        let last_public_type = r.read_u32::<LittleEndian>()?;
        let key_strings = r.read_u32::<LittleEndian>()?;
        let last_public_key = r.read_u32::<LittleEndian>()?;
        let type_id_offset = r.read_u32::<LittleEndian>()?;
        Ok(Self {
            id,
            name,
            type_strings,
            last_public_type,
            key_strings,
            last_public_key,
            type_id_offset,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.id)?;
        let mut name = [0; 128];
        for (i, c) in self.name.encode_utf16().enumerate() {
            name[i] = c;
        }
        for c in name {
            w.write_u16::<LittleEndian>(c)?;
        }
        w.write_u32::<LittleEndian>(self.type_strings)?;
        w.write_u32::<LittleEndian>(self.last_public_type)?;
        w.write_u32::<LittleEndian>(self.key_strings)?;
        w.write_u32::<LittleEndian>(self.last_public_key)?;
        w.write_u32::<LittleEndian>(self.type_id_offset)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResTableTypeSpecHeader {
    /// The type identifier this chunk is holding. Type IDs start
    /// at 1 (corresponding to the value of the type bits in a
    /// resource identifier). 0 is invalid.
    pub id: u8,
    /// Must be 0.
    pub res0: u8,
    /// Must be 0.
    pub res1: u16,
    /// Number of u32 entry configuration masks that follow.
    pub entry_count: u32,
}

impl ResTableTypeSpecHeader {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let id = r.read_u8()?;
        let res0 = r.read_u8()?;
        let res1 = r.read_u16::<LittleEndian>()?;
        let entry_count = r.read_u32::<LittleEndian>()?;
        Ok(Self {
            id,
            res0,
            res1,
            entry_count,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id)?;
        w.write_u8(self.res0)?;
        w.write_u16::<LittleEndian>(self.res1)?;
        w.write_u32::<LittleEndian>(self.entry_count)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResTableTypeHeader {
    /// The type identifier this chunk is holding. Type IDs start
    /// at 1 (corresponding to the value of the type bits in a
    /// resource identifier). 0 is invalid.
    pub id: u8,
    /// Must be 0.
    pub res0: u8,
    /// Must be 0.
    pub res1: u16,
    /// Number of u32 entry indices that follow.
    pub entry_count: u32,
    /// Offset from header where ResTableEntry data starts.
    pub entries_start: u32,
    /// Configuration this collection of entries is designed for.
    pub config: ResTableConfig,
}

impl ResTableTypeHeader {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let id = r.read_u8()?;
        let res0 = r.read_u8()?;
        let res1 = r.read_u16::<LittleEndian>()?;
        let entry_count = r.read_u32::<LittleEndian>()?;
        let entries_start = r.read_u32::<LittleEndian>()?;
        let config = ResTableConfig::read(r)?;
        Ok(Self {
            id,
            res0,
            res1,
            entry_count,
            entries_start,
            config,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id)?;
        w.write_u8(self.res0)?;
        w.write_u16::<LittleEndian>(self.res1)?;
        w.write_u32::<LittleEndian>(self.entry_count)?;
        w.write_u32::<LittleEndian>(self.entries_start)?;
        self.config.write(w)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResTableConfig {
    pub size: u32,
    pub imsi: u32,
    pub locale: u32,
    pub screen_type: ScreenType,
    pub input: u32,
    pub screen_size: u32,
    pub version: u32,
    pub unknown: Vec<u8>,
}

impl ResTableConfig {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let size = r.read_u32::<LittleEndian>()?;
        let imsi = r.read_u32::<LittleEndian>()?;
        let locale = r.read_u32::<LittleEndian>()?;
        let screen_type = ScreenType::read(r)?;
        let input = r.read_u32::<LittleEndian>()?;
        let screen_size = r.read_u32::<LittleEndian>()?;
        let version = r.read_u32::<LittleEndian>()?;
        let unknown_len = size as usize - 28;
        let mut unknown = vec![0; unknown_len];
        r.read_exact(&mut unknown)?;
        Ok(Self {
            size,
            imsi,
            locale,
            screen_type,
            input,
            screen_size,
            version,
            unknown,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.size)?;
        w.write_u32::<LittleEndian>(self.imsi)?;
        w.write_u32::<LittleEndian>(self.locale)?;
        self.screen_type.write(w)?;
        w.write_u32::<LittleEndian>(self.input)?;
        w.write_u32::<LittleEndian>(self.screen_size)?;
        w.write_u32::<LittleEndian>(self.version)?;
        w.write_all(&self.unknown)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenType {
    pub orientation: u8,
    pub touchscreen: u8,
    pub density: u16,
}

impl ScreenType {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let orientation = r.read_u8()?;
        let touchscreen = r.read_u8()?;
        let density = r.read_u16::<LittleEndian>()?;
        Ok(Self {
            orientation,
            touchscreen,
            density,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.orientation)?;
        w.write_u8(self.touchscreen)?;
        w.write_u16::<LittleEndian>(self.density)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResTableEntry {
    pub size: u16,
    pub flags: u16,
    pub key: u32,
    pub value: ResTableValue,
}

impl ResTableEntry {
    pub fn is_complex(&self) -> bool {
        self.flags & 0x1 > 0
    }

    pub fn is_public(&self) -> bool {
        self.flags & 0x2 > 0
    }

    pub fn read(r: &mut impl Read) -> Result<Self> {
        let size = r.read_u16::<LittleEndian>()?;
        let flags = r.read_u16::<LittleEndian>()?;
        let key = r.read_u32::<LittleEndian>()?;
        let is_complex = flags & 0x1 > 0;
        if is_complex {
            debug_assert_eq!(size, 16);
        } else {
            debug_assert_eq!(size, 8);
        }
        let value = ResTableValue::read(r, is_complex)?;
        Ok(Self {
            size,
            flags,
            key,
            value,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u16::<LittleEndian>(self.size)?;
        w.write_u16::<LittleEndian>(self.flags)?;
        w.write_u32::<LittleEndian>(self.key)?;
        self.value.write(w)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResTableValue {
    Simple(ResValue),
    Complex(ResTableMapEntry, Vec<ResTableMap>),
}

impl ResTableValue {
    pub fn read(r: &mut impl Read, is_complex: bool) -> Result<Self> {
        let res = if is_complex {
            let entry = ResTableMapEntry::read(r)?;
            let mut map = Vec::with_capacity(entry.count as usize);
            for _ in 0..entry.count {
                map.push(ResTableMap::read(r)?);
            }
            Self::Complex(entry, map)
        } else {
            Self::Simple(ResValue::read(r)?)
        };
        Ok(res)
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        match self {
            Self::Simple(value) => value.write(w)?,
            Self::Complex(entry, map) => {
                entry.write(w)?;
                for entry in map {
                    entry.write(w)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResValue {
    pub size: u16,
    pub res0: u8,
    pub data_type: u8,
    pub data: u32,
}

impl ResValue {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let size = r.read_u16::<LittleEndian>()?;
        debug_assert_eq!(size, 8);
        let res0 = r.read_u8()?;
        let data_type = r.read_u8()?;
        let data = r.read_u32::<LittleEndian>()?;
        Ok(Self {
            size,
            res0,
            data_type,
            data,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u16::<LittleEndian>(self.size)?;
        w.write_u8(self.res0)?;
        w.write_u8(self.data_type)?;
        w.write_u32::<LittleEndian>(self.data)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ResValueType {
    Null = 0x00,
    Reference = 0x01,
    Attribute = 0x02,
    String = 0x03,
    Float = 0x04,
    Dimension = 0x05,
    Fraction = 0x06,
    IntDec = 0x10,
    IntHex = 0x11,
    IntBoolean = 0x12,
    IntColorArgb8 = 0x1c,
    IntColorRgb8 = 0x1d,
    IntColorArgb4 = 0x1e,
    IntColorRgb4 = 0x1f,
}

impl ResValueType {
    pub fn from_u8(ty: u8) -> Option<Self> {
        Some(match ty {
            x if x == Self::Null as u8 => Self::Null,
            x if x == Self::Reference as u8 => Self::Reference,
            x if x == Self::Attribute as u8 => Self::Attribute,
            x if x == Self::String as u8 => Self::String,
            x if x == Self::Float as u8 => Self::Float,
            x if x == Self::Dimension as u8 => Self::Dimension,
            x if x == Self::Fraction as u8 => Self::Fraction,
            x if x == Self::IntDec as u8 => Self::IntDec,
            x if x == Self::IntHex as u8 => Self::IntHex,
            x if x == Self::IntBoolean as u8 => Self::IntBoolean,
            x if x == Self::IntColorArgb8 as u8 => Self::IntColorArgb8,
            x if x == Self::IntColorRgb8 as u8 => Self::IntColorRgb8,
            x if x == Self::IntColorArgb4 as u8 => Self::IntColorArgb4,
            x if x == Self::IntColorRgb4 as u8 => Self::IntColorRgb4,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ResAttributeType {
    Any = 0x0000_ffff,
    Reference = 1 << 0,
    String = 1 << 1,
    Integer = 1 << 2,
    Boolean = 1 << 3,
    Color = 1 << 4,
    Float = 1 << 5,
    Dimension = 1 << 6,
    Fraction = 1 << 7,
    Enum = 1 << 16,
    Flags = 1 << 17,
}

impl ResAttributeType {
    pub fn from_u32(ty: u32) -> Option<Self> {
        Some(match ty {
            x if x == Self::Any as u32 => Self::Any,
            x if x == Self::Reference as u32 => Self::Reference,
            x if x == Self::String as u32 => Self::String,
            x if x == Self::Integer as u32 => Self::Integer,
            x if x == Self::Boolean as u32 => Self::Boolean,
            x if x == Self::Color as u32 => Self::Color,
            x if x == Self::Float as u32 => Self::Float,
            x if x == Self::Dimension as u32 => Self::Dimension,
            x if x == Self::Fraction as u32 => Self::Fraction,
            x if x == Self::Enum as u32 => Self::Enum,
            x if x == Self::Flags as u32 => Self::Flags,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResTableMapEntry {
    pub parent: u32,
    pub count: u32,
}

impl ResTableMapEntry {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let parent = r.read_u32::<LittleEndian>()?;
        let count = r.read_u32::<LittleEndian>()?;
        Ok(Self { parent, count })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.parent)?;
        w.write_u32::<LittleEndian>(self.count)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResTableMap {
    pub name: u32,
    pub value: ResValue,
}

impl ResTableMap {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let name = r.read_u32::<LittleEndian>()?;
        let value = ResValue::read(r)?;
        Ok(Self { name, value })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.name)?;
        self.value.write(w)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResSpan {
    pub name: i32,
    pub first_char: u32,
    pub last_char: u32,
}

impl ResSpan {
    pub fn read(r: &mut impl Read) -> Result<Option<Self>> {
        let name = r.read_i32::<LittleEndian>()?;
        if name == -1 {
            return Ok(None);
        }
        let first_char = r.read_u32::<LittleEndian>()?;
        let last_char = r.read_u32::<LittleEndian>()?;
        Ok(Some(Self {
            name,
            first_char,
            last_char,
        }))
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.name)?;
        w.write_u32::<LittleEndian>(self.first_char)?;
        w.write_u32::<LittleEndian>(self.last_char)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Chunk {
    Null,
    StringPool(Vec<String>, Vec<Vec<ResSpan>>),
    Table(ResTableHeader, Vec<Chunk>),
    Xml(Vec<Chunk>),
    XmlStartNamespace(ResXmlNodeHeader, ResXmlNamespace),
    XmlEndNamespace(ResXmlNodeHeader, ResXmlNamespace),
    XmlStartElement(ResXmlNodeHeader, ResXmlStartElement, Vec<ResXmlAttribute>),
    XmlEndElement(ResXmlNodeHeader, ResXmlEndElement),
    XmlResourceMap(Vec<u32>),
    TablePackage(ResTablePackageHeader, Vec<Chunk>),
    TableType(ResTableTypeHeader, Vec<u32>, Vec<Option<ResTableEntry>>),
    TableTypeSpec(ResTableTypeSpecHeader, Vec<u32>),
    Unknown,
}

impl Chunk {
    pub fn parse<R: Read + Seek>(r: &mut R) -> Result<Self> {
        let start_pos = r.stream_position()?;
        let header = ResChunkHeader::read(r)?;
        let end_pos = start_pos + header.size as u64;
        match ChunkType::from_u16(header.ty) {
            Some(ChunkType::Null) => {
                tracing::trace!("null");
                Ok(Chunk::Null)
            }
            Some(ChunkType::StringPool) => {
                tracing::trace!("string pool");
                let string_pool_header = ResStringPoolHeader::read(r)?;
                let count =
                    string_pool_header.string_count as i64 + string_pool_header.style_count as i64;
                r.seek(SeekFrom::Current(count * 4))?;
                /*let mut string_indices = Vec::with_capacity(string_pool_header.string_count);
                for _ in 0..string_pool_header.string_count {
                    string_indices.push(r.read_u32::<LittleEndian>()?);
                }
                let mut style_indices = Vec::with_capacity(string_pool_header.style_count);
                for _ in 0..string_pool_header.style_count {
                    style_indices.push(r.read_u32::<LittleEndian>()?);
                }*/
                let mut strings = Vec::with_capacity(string_pool_header.string_count as usize);
                for _ in 0..string_pool_header.string_count {
                    if string_pool_header.is_utf8() {
                        let charsh = r.read_u8()? as u16;
                        let _chars = if charsh > 0x7f {
                            charsh & 0x7f | r.read_u8()? as u16
                        } else {
                            charsh
                        };
                        let bytesh = r.read_u8()? as u16;
                        let bytes = if bytesh > 0x7f {
                            bytesh & 0x7f | r.read_u8()? as u16
                        } else {
                            bytesh
                        };
                        let mut buf = vec![0; bytes as usize];
                        r.read_exact(&mut buf)?;
                        // some times there is an invalid string?
                        let s = String::from_utf8(buf).unwrap_or_default();
                        strings.push(s);
                        if r.read_u8()? != 0 {
                            // fails to read some files otherwise
                            r.seek(SeekFrom::Start(end_pos))?;
                        }
                    } else {
                        let charsh = r.read_u16::<LittleEndian>()? as u32;
                        let chars = if charsh > 0x7fff {
                            charsh & 0x7fff | r.read_u16::<LittleEndian>()? as u32
                        } else {
                            charsh
                        };
                        let mut buf = Vec::with_capacity(chars as usize * 2);
                        loop {
                            let code = r.read_u16::<LittleEndian>()?;
                            if code != 0 {
                                buf.push(code);
                            } else {
                                break;
                            }
                        }
                        let s = String::from_utf16(buf.as_slice())?;
                        strings.push(s);
                    }
                }
                let pos = r.stream_position()? as i64;
                if pos % 4 != 0 {
                    r.seek(SeekFrom::Current(4 - pos % 4))?;
                }
                let mut styles = Vec::with_capacity(string_pool_header.style_count as usize);
                for _ in 0..string_pool_header.style_count {
                    let mut spans = vec![];
                    while let Some(span) = ResSpan::read(r)? {
                        spans.push(span);
                    }
                    styles.push(spans);
                }
                // FIXME: skip some unparsable parts
                r.seek(SeekFrom::Start(end_pos))?;
                Ok(Chunk::StringPool(strings, styles))
            }
            Some(ChunkType::Table) => {
                tracing::trace!("table");
                let table_header = ResTableHeader::read(r)?;
                let mut chunks = vec![];
                while r.stream_position()? < end_pos {
                    chunks.push(Chunk::parse(r)?);
                }
                Ok(Chunk::Table(table_header, chunks))
            }
            Some(ChunkType::Xml) => {
                tracing::trace!("xml");
                let mut chunks = vec![];
                while r.stream_position()? < end_pos {
                    chunks.push(Chunk::parse(r)?);
                }
                Ok(Chunk::Xml(chunks))
            }
            Some(ChunkType::XmlStartNamespace) => {
                tracing::trace!("xml start namespace");
                let node_header = ResXmlNodeHeader::read(r)?;
                let namespace = ResXmlNamespace::read(r)?;
                Ok(Chunk::XmlStartNamespace(node_header, namespace))
            }
            Some(ChunkType::XmlEndNamespace) => {
                tracing::trace!("xml end namespace");
                let node_header = ResXmlNodeHeader::read(r)?;
                let namespace = ResXmlNamespace::read(r)?;
                Ok(Chunk::XmlEndNamespace(node_header, namespace))
            }
            Some(ChunkType::XmlStartElement) => {
                tracing::trace!("xml start element");
                let node_header = ResXmlNodeHeader::read(r)?;
                let start_element = ResXmlStartElement::read(r)?;
                let mut attributes = Vec::with_capacity(start_element.attribute_count as usize);
                for _ in 0..start_element.attribute_count {
                    attributes.push(ResXmlAttribute::read(r)?);
                }
                Ok(Chunk::XmlStartElement(
                    node_header,
                    start_element,
                    attributes,
                ))
            }
            Some(ChunkType::XmlEndElement) => {
                tracing::trace!("xml end element");
                let node_header = ResXmlNodeHeader::read(r)?;
                let end_element = ResXmlEndElement::read(r)?;
                Ok(Chunk::XmlEndElement(node_header, end_element))
            }
            Some(ChunkType::XmlResourceMap) => {
                tracing::trace!("xml resource map");
                let mut resource_map =
                    Vec::with_capacity((header.size as usize - header.header_size as usize) / 4);
                for _ in 0..resource_map.capacity() {
                    resource_map.push(r.read_u32::<LittleEndian>()?);
                }
                Ok(Chunk::XmlResourceMap(resource_map))
            }
            Some(ChunkType::TablePackage) => {
                tracing::trace!("table package");
                let package_header = ResTablePackageHeader::read(r)?;
                let mut chunks = vec![];
                while r.stream_position()? < end_pos {
                    chunks.push(Chunk::parse(r)?);
                }
                Ok(Chunk::TablePackage(package_header, chunks))
            }
            Some(ChunkType::TableType) => {
                tracing::trace!("table type");
                let type_header = ResTableTypeHeader::read(r)?;
                let mut index = Vec::with_capacity(type_header.entry_count as usize);
                for _ in 0..type_header.entry_count {
                    let entry = r.read_u32::<LittleEndian>()?;
                    index.push(entry);
                }
                let mut entries = Vec::with_capacity(type_header.entry_count as usize);
                for offset in &index {
                    if *offset == 0xffff_ffff {
                        entries.push(None);
                    } else {
                        let entry = ResTableEntry::read(r)?;
                        entries.push(Some(entry));
                    }
                }
                Ok(Chunk::TableType(type_header, index, entries))
            }
            Some(ChunkType::TableTypeSpec) => {
                tracing::trace!("table type spec");
                let type_spec_header = ResTableTypeSpecHeader::read(r)?;
                let mut type_spec = vec![0; type_spec_header.entry_count as usize];
                for c in type_spec.iter_mut() {
                    *c = r.read_u32::<LittleEndian>()?;
                }
                Ok(Chunk::TableTypeSpec(type_spec_header, type_spec))
            }
            Some(ChunkType::Unknown) => {
                tracing::trace!("unknown");
                // FIXME: skip some unparsable parts
                r.seek(SeekFrom::Start(end_pos))?;
                Ok(Chunk::Unknown)
            }
            None => {
                anyhow::bail!("unrecognized chunk {:?}", header);
            }
        }
    }

    pub fn write<W: Seek + Write>(&self, w: &mut W) -> Result<()> {
        struct ChunkWriter {
            ty: ChunkType,
            start_chunk: u64,
            end_header: u64,
        }
        impl ChunkWriter {
            fn start_chunk<W: Seek + Write>(ty: ChunkType, w: &mut W) -> Result<Self> {
                let start_chunk = w.stream_position()?;
                ResChunkHeader::default().write(w)?;
                Ok(Self {
                    ty,
                    start_chunk,
                    end_header: 0,
                })
            }

            fn end_header<W: Seek + Write>(&mut self, w: &mut W) -> Result<()> {
                self.end_header = w.stream_position()?;
                Ok(())
            }

            fn end_chunk<W: Seek + Write>(self, w: &mut W) -> Result<(u64, u64)> {
                assert_ne!(self.end_header, 0);
                let end_chunk = w.stream_position()?;
                let header = ResChunkHeader {
                    ty: self.ty as u16,
                    header_size: (self.end_header - self.start_chunk) as u16,
                    size: (end_chunk - self.start_chunk) as u32,
                };
                w.seek(SeekFrom::Start(self.start_chunk))?;
                header.write(w)?;
                w.seek(SeekFrom::Start(end_chunk))?;
                Ok((self.start_chunk, end_chunk))
            }
        }
        match self {
            Chunk::Null => {}
            Chunk::StringPool(strings, styles) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::StringPool, w)?;
                ResStringPoolHeader::default().write(w)?;
                chunk.end_header(w)?;
                let indices_count = strings.len() + styles.len();
                let mut indices = Vec::with_capacity(indices_count);
                for _ in 0..indices_count {
                    w.write_u32::<LittleEndian>(0)?;
                }
                let strings_start = w.stream_position()?;
                for string in strings {
                    indices.push(w.stream_position()? - strings_start);
                    assert!(string.len() < 0x7f);
                    let chars = string.chars().count();
                    w.write_u8(chars as u8)?;
                    w.write_u8(string.len() as u8)?;
                    w.write_all(string.as_bytes())?;
                    w.write_u8(0)?;
                }
                while w.stream_position()? % 4 != 0 {
                    w.write_u8(0)?;
                }
                let styles_start = w.stream_position()?;
                for style in styles {
                    indices.push(w.stream_position()? - styles_start);
                    for span in style {
                        span.write(w)?;
                    }
                    w.write_i32::<LittleEndian>(-1)?;
                }
                let (start_chunk, end_chunk) = chunk.end_chunk(w)?;

                w.seek(SeekFrom::Start(start_chunk + 8))?;
                ResStringPoolHeader {
                    string_count: strings.len() as u32,
                    style_count: styles.len() as u32,
                    flags: ResStringPoolHeader::UTF8_FLAG,
                    strings_start: (strings_start - start_chunk) as u32,
                    styles_start: (styles_start - start_chunk) as u32,
                }
                .write(w)?;
                for index in indices {
                    w.write_u32::<LittleEndian>(index as u32)?;
                }
                w.seek(SeekFrom::Start(end_chunk))?;
            }
            Chunk::Table(table_header, chunks) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::Table, w)?;
                table_header.write(w)?;
                chunk.end_header(w)?;
                for chunk in chunks {
                    chunk.write(w)?;
                }
                chunk.end_chunk(w)?;
            }
            Chunk::Xml(chunks) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::Xml, w)?;
                chunk.end_header(w)?;
                for chunk in chunks {
                    chunk.write(w)?;
                }
                chunk.end_chunk(w)?;
            }
            Chunk::XmlStartNamespace(node_header, namespace) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::XmlStartNamespace, w)?;
                node_header.write(w)?;
                chunk.end_header(w)?;
                namespace.write(w)?;
                chunk.end_chunk(w)?;
            }
            Chunk::XmlEndNamespace(node_header, namespace) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::XmlEndNamespace, w)?;
                node_header.write(w)?;
                chunk.end_header(w)?;
                namespace.write(w)?;
                chunk.end_chunk(w)?;
            }
            Chunk::XmlStartElement(node_header, start_element, attributes) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::XmlStartElement, w)?;
                node_header.write(w)?;
                chunk.end_header(w)?;
                start_element.write(w)?;
                for attr in attributes {
                    attr.write(w)?;
                }
                chunk.end_chunk(w)?;
            }
            Chunk::XmlEndElement(node_header, end_element) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::XmlEndElement, w)?;
                node_header.write(w)?;
                chunk.end_header(w)?;
                end_element.write(w)?;
                chunk.end_chunk(w)?;
            }
            Chunk::XmlResourceMap(resource_map) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::XmlResourceMap, w)?;
                chunk.end_header(w)?;
                for entry in resource_map {
                    w.write_u32::<LittleEndian>(*entry)?;
                }
                chunk.end_chunk(w)?;
            }
            Chunk::TablePackage(package_header, chunks) => {
                let package_start = w.stream_position()?;
                let mut chunk = ChunkWriter::start_chunk(ChunkType::TablePackage, w)?;
                let mut package_header = package_header.clone();
                let header_start = w.stream_position()?;
                package_header.write(w)?;
                chunk.end_header(w)?;

                let type_strings_start = w.stream_position()?;
                package_header.type_strings = (type_strings_start - package_start) as u32;
                chunks[0].write(w)?;

                let key_strings_start = w.stream_position()?;
                package_header.key_strings = (key_strings_start - package_start) as u32;
                chunks[1].write(w)?;

                for chunk in &chunks[2..] {
                    chunk.write(w)?;
                }
                chunk.end_chunk(w)?;

                let end = w.stream_position()?;
                w.seek(SeekFrom::Start(header_start))?;
                package_header.write(w)?;
                w.seek(SeekFrom::Start(end))?;
            }
            Chunk::TableType(type_header, index, entries) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::TableType, w)?;
                type_header.write(w)?;
                chunk.end_header(w)?;
                for offset in index {
                    w.write_u32::<LittleEndian>(*offset)?;
                }
                for entry in entries.iter().flatten() {
                    entry.write(w)?;
                }
                chunk.end_chunk(w)?;
            }
            Chunk::TableTypeSpec(type_spec_header, type_spec) => {
                let mut chunk = ChunkWriter::start_chunk(ChunkType::TableTypeSpec, w)?;
                type_spec_header.write(w)?;
                chunk.end_header(w)?;
                for spec in type_spec {
                    w.write_u32::<LittleEndian>(*spec)?;
                }
                chunk.end_chunk(w)?;
            }
            Chunk::Unknown => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, Cursor};
    use std::path::Path;
    use zip::ZipArchive;

    #[test]
    fn test_parse_android_resources() -> Result<()> {
        crate::tests::init_logger();
        let home = std::env::var("ANDROID_HOME")?;
        let platforms = Path::new(&home).join("platforms");
        for entry in std::fs::read_dir(platforms)? {
            let platform = entry?;
            let android = platform.path().join("android.jar");
            if !android.exists() {
                continue;
            }
            let mut zip = ZipArchive::new(BufReader::new(File::open(&android)?))?;
            let mut f = zip.by_name("resources.arsc")?;
            let mut buf = vec![];
            f.read_to_end(&mut buf)?;
            let mut cursor = Cursor::new(&buf);
            tracing::info!("parsing {}", android.display());
            Chunk::parse(&mut cursor)?;
        }
        Ok(())
    }
}
