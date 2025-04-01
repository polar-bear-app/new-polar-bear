use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Doctor {
    groups: Vec<Group>,
}

impl Default for Doctor {
    fn default() -> Self {
        Self {
            groups: vec![
                Group {
                    name: "clang/llvm toolchain",
                    checks: vec![
                        Check::new("clang", Some(VersionCheck::new("--version", 0, 2))),
                        Check::new("clang++", Some(VersionCheck::new("--version", 0, 2))),
                        Check::new("llvm-ar", None),
                        Check::new("llvm-lib", None),
                        Check::new("llvm-readobj", Some(VersionCheck::new("--version", 1, 4))),
                        Check::new("lld", Some(VersionCheck::new("-flavor ld --version", 0, 1))),
                        Check::new("lld-link", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("lldb", Some(VersionCheck::new("--version", 0, 2))),
                        Check::new("lldb-server", None),
                    ],
                },
                Group {
                    name: "rust",
                    checks: vec![
                        Check::new("rustup", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("cargo", Some(VersionCheck::new("--version", 0, 1))),
                    ],
                },
                Group {
                    name: "android",
                    checks: vec![
                        Check::new("adb", Some(VersionCheck::new("--version", 0, 4))),
                        Check::new("javac", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("java", Some(VersionCheck::new("--version", 0, 1))),
                        Check::new("kotlin", Some(VersionCheck::new("-version", 0, 2))),
                        Check::new("gradle", Some(VersionCheck::new("--version", 2, 1))),
                    ],
                },
                Group {
                    name: "ios",
                    checks: vec![
                        Check::new("idevice_id", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("ideviceinfo", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("ideviceinstaller", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("ideviceimagemounter", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new("idevicedebug", Some(VersionCheck::new("-v", 0, 1))),
                        Check::new(
                            "idevicedebugserverproxy",
                            Some(VersionCheck::new("-v", 0, 1)),
                        ),
                    ],
                },
                Group {
                    name: "linux",
                    checks: vec![Check::new(
                        "mksquashfs",
                        Some(VersionCheck::new("-version", 0, 2)),
                    )],
                },
            ],
        }
    }
}

impl std::fmt::Display for Doctor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for group in &self.groups {
            write!(f, "{}", group)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Group {
    name: &'static str,
    checks: Vec<Check>,
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:-^1$}", self.name, 60)?;
        for check in &self.checks {
            write!(f, "{:20} ", check.name())?;
            if let Ok(path) = check.path() {
                let version = if let Ok(Some(version)) = check.version() {
                    version
                } else {
                    "unknown".into()
                };
                write!(f, "{:20}", version)?;
                write!(f, "{}", path.display())?;
            } else {
                write!(f, "not found")?;
            }
            writeln!(f)?;
        }
        writeln!(f)
    }
}

#[derive(Clone, Copy, Debug)]
struct Check {
    name: &'static str,
    version: Option<VersionCheck>,
}

impl Check {
    pub const fn new(name: &'static str, version: Option<VersionCheck>) -> Self {
        Self { name, version }
    }
}

#[derive(Clone, Copy, Debug)]
struct VersionCheck {
    arg: &'static str,
    row: u8,
    col: u8,
}

impl VersionCheck {
    pub const fn new(arg: &'static str, row: u8, col: u8) -> Self {
        Self { arg, row, col }
    }
}

impl Check {
    fn name(self) -> &'static str {
        self.name
    }

    fn path(self) -> Result<PathBuf> {
        Ok(which::which(self.name)?)
    }

    fn version(self) -> Result<Option<String>> {
        if let Some(version) = self.version {
            let output = Command::new(self.name)
                .args(version.arg.split(' '))
                .output()?;
            anyhow::ensure!(output.status.success(), "failed to run {}", self.name);
            let output = std::str::from_utf8(&output.stdout)?;
            if let Some(line) = output.lines().nth(version.row as _) {
                let mut col = version.col as usize;
                if line.starts_with("Apple ") || line.starts_with("Homebrew ") {
                    col += 1;
                }
                if let Some(col) = line.split(' ').nth(col) {
                    return Ok(Some(col.to_string()));
                }
            }
            anyhow::bail!("failed to parse version: {:?}", output);
        } else {
            Ok(None)
        }
    }
}

pub fn doctor() {
    let doctor = Doctor::default();
    print!("{}", doctor);
}
