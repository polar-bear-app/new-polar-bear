use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename = "metadata")]
pub struct Metadata {
    #[serde(rename = "$unflatten=versioning")]
    versioning: Versioning,
}

impl Metadata {
    pub fn latest(&self) -> &str {
        &self.versioning.latest
    }

    pub fn release(&self) -> &str {
        &self.versioning.release
    }

    pub fn versions(&self) -> &[String] {
        &self.versioning.versions.versions
    }
}

#[derive(Deserialize, Serialize)]
struct Versioning {
    latest: String,
    release: String,
    #[serde(rename = "$unflatten=versions")]
    versions: Versions,
}

#[derive(Deserialize, Serialize)]
struct Versions {
    #[serde(rename = "$unflatten=version")]
    #[serde(default)]
    versions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_metadata() -> Result<()> {
        let meta = r#"
            <metadata>
                <groupId>group</groupId>
                <artifactId>artifact</artifactId>
                <versioning>
                    <latest>b</latest>
                    <release>b</release>
                    <versions>
                        <version>a</version>
                        <version>b</version>
                    </versions>
                    <lastUpdated>xxxx</lastUpdated>
                </versioning>
            </metadata>"#;
        let meta: Metadata = quick_xml::de::from_str(meta)?;
        assert_eq!(meta.versions().len(), 2);
        assert_eq!(meta.versions()[0], "a");
        assert_eq!(meta.versions()[1], "b");
        assert_eq!(meta.latest(), "b");
        assert_eq!(meta.release(), "b");
        Ok(())
    }
}
