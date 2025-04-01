use anyhow::{ensure, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Span {
    offset: usize,
    length: usize,
}

impl Span {
    pub fn start(self) -> usize {
        self.offset
    }

    pub fn end(self) -> usize {
        self.offset + self.length
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct DataItem {
    string_spans: Vec<Span>,
    string_data: Vec<u8>,
    blob_spans: Vec<Span>,
    blob_data: Vec<u8>,
}

impl DataItem {
    pub const IDENTIFIER: &'static [u8; 16] = b"[mrm_dataitem] \0";

    pub fn read(r: &mut impl Read) -> Result<Self> {
        ensure!(r.read_u32::<LE>()? == 0);
        let num_strings = r.read_u16::<LE>()? as usize;
        let num_blobs = r.read_u16::<LE>()? as usize;
        let total_data_length = r.read_u32::<LE>()? as usize;
        let mut string_spans = Vec::with_capacity(num_strings);
        for _ in 0..num_strings {
            let offset = r.read_u16::<LE>()? as usize;
            let length = r.read_u16::<LE>()? as usize;
            string_spans.push(Span { offset, length });
        }
        let string_data_length = if let Some(span) = string_spans.last() {
            span.end()
        } else {
            0
        };
        let mut blob_spans = Vec::with_capacity(num_blobs);
        for _ in 0..num_blobs {
            let offset = r.read_u32::<LE>()? as usize - string_data_length;
            let length = r.read_u32::<LE>()? as usize;
            blob_spans.push(Span { offset, length });
        }
        let blob_data_length = total_data_length - string_data_length;
        let mut string_data = Vec::with_capacity(string_data_length);
        r.take(string_data_length as u64)
            .read_to_end(&mut string_data)?;
        let mut blob_data = Vec::with_capacity(blob_data_length);
        r.take(blob_data_length as u64)
            .read_to_end(&mut blob_data)?;
        Ok(Self {
            string_spans,
            string_data,
            blob_spans,
            blob_data,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LE>(0)?;
        w.write_u16::<LE>(self.string_spans.len() as u16)?;
        w.write_u16::<LE>(self.blob_spans.len() as u16)?;
        w.write_u32::<LE>((self.string_data.len() + self.blob_data.len()) as u32)?;
        for span in &self.string_spans {
            w.write_u16::<LE>(span.offset as _)?;
            w.write_u16::<LE>(span.length as _)?;
        }
        let offset = self.string_data.len() as u32;
        for span in &self.blob_spans {
            w.write_u32::<LE>(span.offset as u32 + offset)?;
            w.write_u32::<LE>(span.length as u32)?;
        }
        w.write_all(&self.string_data)?;
        w.write_all(&self.blob_data)?;
        Ok(())
    }

    pub fn num_strings(&self) -> usize {
        self.string_spans.len()
    }

    pub fn string(&self, index: usize) -> Option<&str> {
        let span = self.string_spans.get(index)?;
        let bytes = &self.string_data[span.start()..(span.end() - 1)];
        std::str::from_utf8(bytes).ok()
    }

    pub fn num_blobs(&self) -> usize {
        self.blob_spans.len()
    }

    pub fn blob(&self, index: usize) -> Option<&[u8]> {
        let span = self.blob_spans.get(index)?;
        Some(&self.blob_data[span.start()..span.end()])
    }

    pub fn add_string(&mut self, s: &str) -> usize {
        let offset = self.string_data.len();
        let length = s.len() + 1;
        let index = self.string_spans.len();
        self.string_spans.push(Span { offset, length });
        self.string_data.extend_from_slice(s.as_bytes());
        self.string_data.push(0);
        index
    }

    pub fn add_blob(&mut self, blob: &[u8]) -> usize {
        let offset = self.blob_data.len();
        let length = blob.len();
        let index = self.blob_spans.len();
        self.blob_spans.push(Span { offset, length });
        self.blob_data.extend_from_slice(blob);
        index
    }
}

impl std::fmt::Debug for DataItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list()
            .entries((0..self.num_strings()).filter_map(|i| self.string(i)))
            .entries((0..self.num_blobs()).filter_map(|i| self.blob(i)))
            .finish()
    }
}
