use anyhow::{Context, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Target {
    ArmV7a = 1,
    Arm64V8a = 2,
    X86 = 3,
    X86_64 = 4,
}

impl Target {
    /// Identifier used in the NDK to refer to the ABI
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Arm64V8a => "arm64-v8a",
            Self::ArmV7a => "armeabi-v7a",
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VersionCode {
    major: u8,
    minor: u8,
    patch: u8,
}

impl VersionCode {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn from_semver(version: &str) -> Result<Self> {
        let mut iter = version.split(|c1| ['.', '-', '+'].iter().any(|c2| c1 == *c2));
        let mut p = || {
            iter.next()
                .context("invalid semver")?
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid semver"))
        };
        Ok(Self::new(p()?, p()?, p()?))
    }

    pub fn to_code(&self, apk_id: u8) -> u32 {
        (apk_id as u32) << 24
            | (self.major as u32) << 16
            | (self.minor as u32) << 8
            | self.patch as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_semver() {
        let v = VersionCode::from_semver("0.0.0").unwrap();
        assert_eq!(v, VersionCode::new(0, 0, 0));
        let v = VersionCode::from_semver("254.254.254-alpha.fix+2").unwrap();
        assert_eq!(v, VersionCode::new(254, 254, 254));
    }
}
