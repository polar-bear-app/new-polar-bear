use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ContentTypesBuilder {
    ext: HashSet<String>,
    inner: Option<ContentTypes>,
}

impl ContentTypesBuilder {
    pub fn add(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            if let Some(ext) = ext.to_str() {
                if !self.ext.contains(ext) {
                    let mime = mime_guess::from_ext(ext).first_or_octet_stream();
                    self.inner.as_mut().unwrap().rules.push(Rule::Default {
                        ext: ext.into(),
                        mime: mime.to_string(),
                    });
                    self.ext.insert(ext.to_string());
                }
            }
        }
    }

    pub fn finish(&mut self) -> ContentTypes {
        self.inner.take().unwrap()
    }
}

impl Default for ContentTypesBuilder {
    fn default() -> Self {
        Self {
            ext: Default::default(),
            inner: Some(Default::default()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename(serialize = "Types"))]
pub struct ContentTypes {
    #[serde(default = "default_namespace")]
    xmlns: String,
    pub rules: Vec<Rule>,
}

impl Default for ContentTypes {
    fn default() -> Self {
        Self {
            xmlns: default_namespace(),
            rules: vec![
                Rule::Override {
                    part_name: "/AppxBlockMap.xml".into(),
                    mime: "application/vnd.ms-appx.blockmap+xml".into(),
                },
                Rule::Override {
                    part_name: "/AppxSignature.p7x".into(),
                    mime: "application/vnd.ms-appx.signature".into(),
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Rule {
    Default {
        #[serde(rename(serialize = "Extension"))]
        ext: String,
        #[serde(rename(serialize = "ContentType"))]
        mime: String,
    },
    Override {
        #[serde(rename(serialize = "PartName"))]
        part_name: String,
        #[serde(rename(serialize = "ContentType"))]
        mime: String,
    },
}

fn default_namespace() -> String {
    "http://schemas.openxmlformats.org/package/2006/content-types".into()
}
