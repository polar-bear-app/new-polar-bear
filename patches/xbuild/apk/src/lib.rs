use crate::compiler::Table;
use crate::res::Chunk;
use anyhow::{Context, Result};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use xcommon::{Scaler, ScalerOpts, Zip, ZipFileOptions};

mod compiler;
pub mod manifest;
pub mod res;
mod sign;
mod utils;

pub use crate::manifest::AndroidManifest;
pub use crate::utils::{Target, VersionCode};
pub use xcommon::{Certificate, Signer};
pub use zip;

pub struct Apk {
    manifest: AndroidManifest,
    path: PathBuf,
    zip: Zip,
}

impl Apk {
    pub fn new(path: PathBuf, manifest: AndroidManifest, compress: bool) -> Result<Self> {
        let zip = Zip::new(&path, compress)?;
        Ok(Self {
            manifest,
            path,
            zip,
        })
    }

    pub fn add_res(&mut self, icon: Option<&Path>, android: &Path) -> Result<()> {
        let mut buf = vec![];
        let mut table = Table::default();
        table.import_apk(android)?;
        if let Some(path) = icon {
            let mut scaler = Scaler::open(path)?;
            scaler.optimize();
            let package = if let Some(package) = self.manifest.package.as_ref() {
                package
            } else {
                anyhow::bail!("missing manifest.package");
            };
            let mipmap = crate::compiler::compile_mipmap(package, "icon")?;

            let mut cursor = Cursor::new(&mut buf);
            mipmap.chunk().write(&mut cursor)?;
            self.zip.create_file(
                Path::new("resources.arsc"),
                ZipFileOptions::Aligned(4),
                &buf,
            )?;

            for (name, size) in mipmap.variants() {
                buf.clear();
                let mut cursor = Cursor::new(&mut buf);
                scaler.write(&mut cursor, ScalerOpts::new(size))?;
                self.zip
                    .create_file(name.as_ref(), ZipFileOptions::Aligned(4), &buf)?;
            }

            table.import_chunk(mipmap.chunk());
            self.manifest.application.icon = Some("@mipmap/icon".into());
        }
        let manifest = crate::compiler::compile_manifest(&self.manifest, &table)?;
        buf.clear();
        let mut cursor = Cursor::new(&mut buf);
        manifest.write(&mut cursor)?;
        self.zip.create_file(
            Path::new("AndroidManifest.xml"),
            ZipFileOptions::Compressed,
            &buf,
        )?;
        Ok(())
    }

    pub fn add_asset(&mut self, asset: &Path, opts: ZipFileOptions) -> Result<()> {
        let file_name = asset
            .file_name()
            .context("Asset must have file_name component")?;
        let dest = Path::new("assets").join(file_name);
        if asset.is_dir() {
            tracing::info!("Embedding asset directory `{}`", asset.display());
            self.zip.add_directory(asset, &dest, opts)
        } else {
            tracing::info!("Embedding asset file `{}`", asset.display());
            self.zip.add_file(asset, &dest, opts)
        }
        .with_context(|| format!("While embedding asset `{}`", asset.display()))
    }

    pub fn add_dex(&mut self, dex: &Path) -> Result<()> {
        self.zip
            .add_file(dex, Path::new("classes.dex"), ZipFileOptions::Compressed)?;
        Ok(())
    }

    pub fn add_lib(&mut self, target: Target, path: &Path) -> Result<()> {
        let name = path.file_name().context("invalid path")?;
        self.zip.add_file(
            path,
            &Path::new("lib").join(target.as_str()).join(name),
            ZipFileOptions::Compressed,
        )
    }

    pub fn finish(self, signer: Option<Signer>) -> Result<()> {
        self.zip.finish()?;
        crate::sign::sign(&self.path, signer)?;
        Ok(())
    }

    pub fn sign(path: &Path, signer: Option<Signer>) -> Result<()> {
        crate::sign::sign(path, signer)
    }

    pub fn verify(path: &Path) -> Result<Vec<Certificate>> {
        crate::sign::verify(path)
    }

    pub fn entry_point(path: &Path) -> Result<EntryPoint> {
        let manifest = xcommon::extract_zip_file(path, "AndroidManifest.xml")?;
        let chunks = if let Chunk::Xml(chunks) = Chunk::parse(&mut Cursor::new(manifest))? {
            chunks
        } else {
            anyhow::bail!("invalid manifest 0");
        };
        let strings = if let Chunk::StringPool(strings, _) = &chunks[0] {
            strings
        } else {
            anyhow::bail!("invalid manifest 1");
        };
        let mut manifest = None;
        let mut package = None;
        let mut activity = None;
        let mut name = None;
        for (i, s) in strings.iter().enumerate() {
            match s.as_str() {
                "manifest" => {
                    manifest = Some(i as i32);
                }
                "package" => {
                    package = Some(i as i32);
                }
                "activity" => {
                    activity = Some(i as i32);
                }
                "name" => {
                    name = Some(i as i32);
                }
                _ => {}
            }
        }
        let (manifest, package, activity, name) =
            if let (Some(manifest), Some(package), Some(activity), Some(name)) =
                (manifest, package, activity, name)
            {
                (manifest, package, activity, name)
            } else {
                anyhow::bail!("invalid manifest 2");
            };
        let mut package_value = None;
        let mut name_value = None;
        for chunk in &chunks[2..] {
            if let Chunk::XmlStartElement(_, el, attrs) = chunk {
                match el.name {
                    x if x == manifest => {
                        package_value = attrs
                            .iter()
                            .find(|attr| attr.name == package)
                            .map(|attr| attr.raw_value);
                    }
                    x if x == activity => {
                        // should really select the activity with the intent-filter
                        // `android.intent.action.MAIN`. for now we settle for the
                        // first activity
                        if name_value.is_some() {
                            continue;
                        }
                        name_value = attrs
                            .iter()
                            .find(|attr| attr.name == name)
                            .map(|attr| attr.raw_value);
                    }
                    _ => {}
                }
            }
        }
        let entry = if let (Some(package_value), Some(name_value)) = (package_value, name_value) {
            EntryPoint {
                package: strings[package_value as usize].clone(),
                activity: strings[name_value as usize].clone(),
            }
        } else {
            anyhow::bail!("invalid manifest 3");
        };
        Ok(entry)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryPoint {
    pub package: String,
    pub activity: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

    pub fn init_logger() {
        tracing_log::LogTracer::init().ok();
        let env = std::env::var(EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| "info".to_owned());
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_span_events(FmtSpan::ACTIVE | FmtSpan::CLOSE)
            .with_env_filter(EnvFilter::new(env))
            .with_writer(std::io::stderr)
            .finish();
        tracing::subscriber::set_global_default(subscriber).ok();
    }

    pub fn find_android_jar() -> Result<PathBuf> {
        let home = std::env::var("ANDROID_HOME")?;
        let platforms = Path::new(&home).join("platforms");
        let mut jar = None;
        for entry in std::fs::read_dir(platforms)? {
            let android = entry?.path().join("android.jar");
            if android.exists() {
                jar = Some(android);
                break;
            }
        }
        Ok(jar.unwrap())
    }

    pub fn android_jar(platform: u16) -> Result<PathBuf> {
        let home = std::env::var("ANDROID_HOME")?;
        let android = Path::new(&home)
            .join("platforms")
            .join(format!("android-{}", platform))
            .join("android.jar");
        Ok(android)
    }
}
