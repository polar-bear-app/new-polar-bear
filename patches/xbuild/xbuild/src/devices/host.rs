use crate::{Arch, Platform};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug)]
pub(crate) struct Host;

impl Host {
    pub fn name(&self) -> Result<String> {
        if cfg!(target_os = "linux") {
            let output = Command::new("uname").output()?;
            anyhow::ensure!(output.status.success(), "uname failed");
            let name = std::str::from_utf8(&output.stdout)?.trim();
            Ok(name.to_string())
        } else {
            Ok("host".to_string())
        }
    }

    pub fn platform(&self) -> Result<Platform> {
        Platform::host()
    }

    pub fn arch(&self) -> Result<Arch> {
        Arch::host()
    }

    pub fn details(&self) -> Result<String> {
        if cfg!(target_os = "linux") {
            let os_release = std::fs::read_to_string("/etc/os-release")?;
            let mut distro = os_release
                .lines()
                .filter_map(|line| line.split_once('='))
                .filter(|(k, _)| *k == "NAME")
                .map(|(_, v)| v.trim_matches('"').to_string())
                .next()
                .unwrap_or_default();
            let output = Command::new("uname").arg("-r").output()?;
            anyhow::ensure!(output.status.success(), "uname failed");
            distro.push(' ');
            distro.push_str(std::str::from_utf8(&output.stdout)?.trim());
            Ok(distro)
        } else {
            Ok("".to_string())
        }
    }

    pub fn run(&self, path: &Path) -> Result<()> {
        Command::new(path).status()?;
        Ok(())
    }

    pub fn lldb(&self, executable: &Path) -> Result<()> {
        Command::new("lldb").arg(executable).status()?;
        Ok(())
    }
}
