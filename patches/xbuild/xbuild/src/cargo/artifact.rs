use std::path::Path;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Artifact {
    Root(String),
    Example(String),
}

impl AsRef<Path> for Artifact {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            Self::Root(_) => "",
            Self::Example(_) => "examples",
        })
    }
}

impl Artifact {
    pub fn name(&self) -> &str {
        match self {
            Self::Root(name) => name,
            Self::Example(name) => name,
        }
    }

    pub fn file_name(&self, ty: CrateType, target: &str) -> String {
        match ty {
            CrateType::Bin => {
                if target.contains("windows") {
                    format!("{}.exe", self.name())
                } else if target.contains("wasm") {
                    format!("{}.wasm", self.name())
                } else {
                    self.name().to_string()
                }
            }
            CrateType::Lib => format!("lib{}.rlib", self.name().replace('-', "_")),
            CrateType::Staticlib => format!("lib{}.a", self.name().replace('-', "_")),
            CrateType::Cdylib => {
                let name = self.name().replace('-', "_");
                if target.contains("windows") {
                    format!("{}.dll", name)
                } else if target.contains("darwin") {
                    format!("lib{}.dylib", name)
                } else {
                    format!("lib{}.so", name)
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CrateType {
    Bin,
    Lib,
    Staticlib,
    Cdylib,
}
