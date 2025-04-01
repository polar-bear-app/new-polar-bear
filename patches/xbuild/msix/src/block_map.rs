use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use zip::read::ZipFile;

pub struct BlockMapBuilder {
    block_map: AppxBlockMap,
    buf: Vec<u8>,
}

impl Default for BlockMapBuilder {
    fn default() -> Self {
        Self {
            block_map: AppxBlockMap::default(),
            buf: Vec::with_capacity(65_536),
        }
    }
}

impl BlockMapBuilder {
    pub fn add(&mut self, f: &mut ZipFile) -> Result<()> {
        let name = Path::new(f.name())
            .iter()
            .map(|seg| seg.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("\\");
        let size = f.size();
        let mut file = File {
            lfh_size: 30 + name.len() as u16,
            name,
            size,
            ..Default::default()
        };
        loop {
            self.buf.clear();
            f.take(self.buf.capacity() as u64)
                .read_to_end(&mut self.buf)?;
            file.blocks.push(Block::new(&self.buf));
            if self.buf.len() != self.buf.capacity() {
                break;
            }
        }
        self.block_map.files.push(file);
        Ok(())
    }

    pub fn finish(self) -> AppxBlockMap {
        self.block_map
    }
}

/// Defines the root element of the app package block map. The BlockMap element
/// specifies the algorithm that is used to compute cryptographic hashes and
/// contains a sequence of File child elements that are associated with each
/// file that is stored in the package.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename(serialize = "BlockMap"))]
pub struct AppxBlockMap {
    #[serde(rename(serialize = "xmlns"))]
    #[serde(default = "default_namespace")]
    ns: String,
    #[serde(rename(serialize = "HashMethod"))]
    #[serde(default = "default_hash_method")]
    hash_method: String,
    /// Files in the package.
    #[serde(rename(serialize = "File"))]
    pub files: Vec<File>,
}

impl Default for AppxBlockMap {
    fn default() -> Self {
        Self {
            ns: default_namespace(),
            hash_method: default_hash_method(),
            files: Default::default(),
        }
    }
}

/// Represents a file contained in the package.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct File {
    /// Root path and file name.
    #[serde(rename(serialize = "Name"))]
    pub name: String,
    /// Size, in bytes, of the file's uncompressed data.
    #[serde(rename(serialize = "Size"))]
    pub size: u64,
    /// Size, in bytes, of the file's Local File Header (LFH) structure in the
    /// package. For more info about file headers, see ZIP file format
    /// specification.
    #[serde(rename(serialize = "LfhSize"))]
    pub lfh_size: u16,
    /// Blocks that make up the file.
    #[serde(rename(serialize = "Block"))]
    pub blocks: Vec<Block>,
}

/// Represents a 64kib block of binary data contained in a file.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Block {
    /// The hash value of the uncompressed data block.
    #[serde(rename(serialize = "Hash"))]
    pub hash: String,
    /// The size, in bytes, of the data block when stored in the package. If
    /// the file data is compressed, the size of each compressed block
    /// potentially varies in size.
    #[serde(rename(serialize = "Size"))]
    pub size: Option<u16>,
}

impl Block {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            hash: base64::encode(Sha256::digest(bytes)),
            size: None,
        }
    }
}

fn default_namespace() -> String {
    "http://schemas.microsoft.com/appx/2010/blockmap".into()
}

fn default_hash_method() -> String {
    "http://www.w3.org/2001/04/xmlenc#sha256".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_map() {
        let mut map = AppxBlockMap::default();
        let mut file = File {
            name: "file.ext".into(),
            size: 12,
            lfh_size: 30,
            blocks: Default::default(),
        };
        file.blocks.push(Block {
            hash: "base64".into(),
            size: Some(12),
        });
        map.files.push(file);
        let _xml = quick_xml::se::to_string(&map).unwrap();
        //println!("{}", xml);
        //assert!(false);
    }
}
