use crate::package::{Package, Version};
use anyhow::{Context, Result};
use pubgrub::range::Range;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Default, Deserialize, Serialize)]
#[serde(rename = "project")]
pub struct Pom {
    packaging: Option<String>,
    #[serde(default)]
    dependencies: Dependencies,
}

impl Pom {
    pub fn packaging(&self) -> &str {
        self.packaging.as_deref().unwrap_or("jar")
    }

    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies.dependencies
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename = "dependencies")]
struct Dependencies {
    #[serde(rename = "$value")]
    #[serde(default)]
    dependencies: Vec<Dependency>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename = "dependency")]
pub struct Dependency {
    #[serde(rename = "$unflatten=groupId")]
    group: String,
    #[serde(rename = "$unflatten=artifactId")]
    name: String,
    #[serde(rename = "$unflatten=version")]
    version: String,
    #[serde(rename = "$unflatten=scope")]
    scope: Option<String>,
}

impl Dependency {
    pub fn package(&self) -> Package {
        Package {
            group: self.group.clone(),
            name: self.name.clone(),
        }
    }

    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    pub fn range(&self) -> Result<Range<Version>> {
        crate::range::range(&self.version)
    }
}

impl FromStr for Dependency {
    type Err = anyhow::Error;

    fn from_str(dep: &str) -> Result<Self> {
        let (group, rest) = dep.split_once(':').context("invalid dep")?;
        let (name, version) = rest.split_once(':').context("invalid dep")?;
        Ok(Self {
            group: group.into(),
            name: name.into(),
            version: version.into(),
            scope: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_dep() -> Result<()> {
        let dep = r#"
            <dependency>
                <groupId>group</groupId>
                <artifactId>name</artifactId>
                <version>1.0.0-alpha</version>
            </dependency>"#;
        let dep: Dependency = quick_xml::de::from_str(dep)?;
        assert_eq!(dep.package().group, "group");
        assert_eq!(dep.package().name, "name");
        //assert_eq!(dep.version()?.to_string(), "1.0.0-alpha");
        Ok(())
    }

    #[test]
    fn test_pom() -> Result<()> {
        let pom = r#"
            <project>
                <dependencies>
                    <dependency>
                        <groupId>group</groupId>
                        <artifactId>name</artifactId>
                        <version>0.0.1</version>
                    </dependency>
                </dependencies>
            </project>"#;
        let pom: Pom = quick_xml::de::from_str(pom)?;
        let deps = pom.dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].package().group, "group");
        assert_eq!(deps[0].package().name, "name");
        //assert_eq!(deps[0].version()?.to_string(), "0.0.1");
        Ok(())
    }

    #[test]
    fn test_pom2() -> Result<()> {
        let pom = r#"
            <project>
                <dependencies/>
            </project>"#;
        let pom: Pom = quick_xml::de::from_str(pom)?;
        let deps = pom.dependencies();
        assert!(deps.is_empty());
        Ok(())
    }
}
