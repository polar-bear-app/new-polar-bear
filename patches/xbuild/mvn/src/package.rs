use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Package {
    #[serde(rename = "$unflatten=groupId")]
    pub group: String,
    #[serde(rename = "$unflatten=artifactId")]
    pub name: String,
}

impl Package {
    pub fn new(group: &str, name: &str) -> Self {
        Self {
            group: group.into(),
            name: name.into(),
        }
    }

    pub fn file_name(&self) -> String {
        format!("{}-{}.metadata.xml", self.group, self.name)
    }

    pub fn url(&self, repo: &str) -> String {
        format!(
            "{repo}/{group}/{name}/maven-metadata.xml",
            repo = repo,
            group = self.group.replace('.', "/"),
            name = self.name,
        )
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.group, self.name)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub suffix: Option<String>,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        if self.major > other.major {
            return Ordering::Greater;
        }
        if other.major > self.major {
            return Ordering::Less;
        }
        if self.minor > other.minor {
            return Ordering::Greater;
        }
        if other.minor > self.minor {
            return Ordering::Less;
        }
        if self.patch > other.patch {
            return Ordering::Greater;
        }
        if other.patch > self.patch {
            return Ordering::Less;
        }
        match (self.suffix.as_ref(), other.suffix.as_ref()) {
            (Some(s1), Some(s2)) => s1.cmp(s2),
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
        }
    }
}

impl pubgrub::version::Version for Version {
    fn lowest() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            suffix: None,
        }
    }

    fn bump(&self) -> Self {
        let patch = if self.suffix.is_some() {
            self.patch
        } else {
            self.patch + 1
        };
        Self {
            major: self.major,
            minor: self.minor,
            patch,
            suffix: None,
        }
    }
}

impl std::str::FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(version: &str) -> Result<Self> {
        let (version, suffix) = version
            .split_once('-')
            .map(|(v, s)| (v, Some(s.to_string())))
            .unwrap_or_else(|| (version, None));
        let mut iter = version.split('.').map(u32::from_str);
        let major = iter.next().transpose()?.unwrap_or_default();
        let minor = iter.next().transpose()?.unwrap_or_default();
        let patch = iter.next().transpose()?.unwrap_or_default();
        Ok(Version {
            major,
            minor,
            patch,
            suffix,
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(suffix) = self.suffix.as_ref() {
            write!(f, "-{}", suffix)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Artifact<'a> {
    pub package: &'a Package,
    pub version: &'a Version,
}

impl Artifact<'_> {
    pub fn file_name(self, ext: &str) -> String {
        format!(
            "{}-{}-{}.{}",
            self.package.group, self.package.name, self.version, ext
        )
    }

    pub fn url(self, repo: &str, ext: &str) -> String {
        format!(
            "{repo}/{group}/{name}/{version}/{name}-{version}.{ext}",
            repo = repo,
            group = self.package.group.replace('.', "/"),
            name = self.package.name,
            version = self.version,
            ext = ext,
        )
    }
}

impl std::fmt::Display for Artifact<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.package.group, self.package.name, self.version,
        )
    }
}
