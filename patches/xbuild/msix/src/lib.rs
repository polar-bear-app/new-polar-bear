use crate::block_map::BlockMapBuilder;
use crate::content_types::ContentTypesBuilder;
use crate::p7x::Digests;
use anyhow::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::{Path, PathBuf};
use xcommon::{Scaler, ScalerOptsBuilder, Signer, Zip, ZipFileOptions, ZipInfo};
use zip::ZipArchive;

mod block_map;
mod content_types;
pub mod manifest;
pub mod p7x;
mod pkcs7;

pub use crate::manifest::AppxManifest;

const DEBUG_PEM: &str = include_str!("../assets/debug.pem");

const IMAGES: [(&str, (u32, u32), f32); 8] = [
    ("SmallTile", (71, 71), 0.34),
    ("Square150x150Logo", (150, 150), 0.34),
    ("Wide310x150Logo", (310, 150), 0.34),
    ("LargeTile", (310, 310), 0.0),
    ("Square44x44Logo", (44, 44), 0.0),
    ("SplashScreen", (620, 300), 0.34),
    ("BadgeLogo", (24, 24), 0.0),
    ("StoreLogo", (50, 50), 0.0),
];

pub struct Msix {
    manifest: AppxManifest,
    path: PathBuf,
    zip: Zip,
    compress: bool,
}

impl Msix {
    pub fn new(path: PathBuf, manifest: AppxManifest, compress: bool) -> Result<Self> {
        Ok(Self {
            manifest,
            zip: Zip::new(&path, compress)?,
            path,
            compress,
        })
    }

    pub fn add_icon(&mut self, path: &Path) -> Result<()> {
        let mut scaler = Scaler::open(path)?;
        scaler.optimize();
        let images = Path::new("Images");
        let mut buf = vec![];
        for (base_name, (width, height), padding) in IMAGES {
            for scale in [1.0, 1.25, 1.5, 2.0, 4.0] {
                buf.clear();
                let opts = ScalerOptsBuilder::new(width, height)
                    .scale(scale)
                    .padding(padding)
                    .build();
                scaler.write(&mut Cursor::new(&mut buf), opts)?;
                let name = format!("{}.scale-{}.png", base_name, (scale * 100.0) as u32);
                self.zip
                    .create_file(&images.join(name), ZipFileOptions::Unaligned, &buf)?;
            }
        }
        Ok(())
    }

    pub fn add_file(&mut self, source: &Path, dest: &Path, opts: ZipFileOptions) -> Result<()> {
        self.zip.add_file(source, dest, opts)
    }

    pub fn add_directory(
        &mut self,
        source: &Path,
        dest: &Path,
        opts: ZipFileOptions,
    ) -> Result<()> {
        self.zip.add_directory(source, dest, opts)
    }

    pub fn finish(mut self, signer: Option<Signer>) -> Result<()> {
        self.zip.create_file(
            "AppxManifest.xml".as_ref(),
            ZipFileOptions::Compressed,
            &to_xml(&self.manifest, true),
        )?;
        self.zip.finish()?;
        Self::sign(&self.path, signer, self.compress)
    }

    pub fn sign(path: &Path, signer: Option<Signer>, compress: bool) -> Result<()> {
        let signer = signer
            .map(Ok)
            .unwrap_or_else(|| Signer::new(DEBUG_PEM))
            .unwrap();

        // add content types and block map
        let mut zip = ZipArchive::new(BufReader::new(File::open(path)?))?;
        let mut content_types = ContentTypesBuilder::default();
        let mut block_map = BlockMapBuilder::default();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            content_types.add(file.name().as_ref());
            block_map.add(&mut file)?;
        }
        let content_types = to_xml(&content_types.finish(), true);
        let axct = Sha256::digest(&content_types);
        let block_map = to_xml(&block_map.finish(), false);
        let axbm = Sha256::digest(&block_map);
        let mut zip = Zip::append(path, compress)?;
        zip.create_file(
            "[Content_Types].xml".as_ref(),
            ZipFileOptions::Compressed,
            &content_types,
        )?;
        zip.create_file(
            "AppxBlockMap.xml".as_ref(),
            ZipFileOptions::Compressed,
            &block_map,
        )?;
        zip.finish()?;

        // compute zip hashes
        let mut r = BufReader::new(File::open(path)?);
        let info = ZipInfo::new(&mut r)?;
        r.rewind()?;
        let mut hasher = Sha256::new();
        let mut pc = (&mut r).take(info.cd_start);
        std::io::copy(&mut pc, &mut hasher)?;
        let axpc = hasher.finalize_reset();
        hasher.reset();
        std::io::copy(&mut r, &mut hasher)?;
        let axcd = hasher.finalize();
        let digests = Digests {
            axpc: axpc.into(),
            axcd: axcd.into(),
            axct: axct.into(),
            axbm: axbm.into(),
            ..Default::default()
        };

        // sign zip
        let sig = p7x::p7x(&signer, &digests);
        let mut zip = Zip::append(path, compress)?;
        zip.create_file(
            "AppxSignature.p7x".as_ref(),
            ZipFileOptions::Compressed,
            &sig,
        )?;
        zip.finish()?;
        Ok(())
    }
}

fn to_xml<T: Serialize>(xml: &T, standalone: bool) -> Vec<u8> {
    let mut buf = vec![];
    let standalone = if standalone { "yes" } else { "no" };
    buf.extend_from_slice(
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="{}"?>"#,
            standalone
        )
        .as_bytes(),
    );
    quick_xml::se::to_writer(&mut buf, xml).unwrap();
    buf
}
