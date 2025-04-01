use crate::devices::{Backend, Device};
use crate::{Arch, BuildEnv, Platform};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub(crate) struct IMobileDevice {
    idevice_id: PathBuf,
    ideviceinfo: PathBuf,
    ideviceimagemounter: PathBuf,
    ideviceinstaller: PathBuf,
    idevicedebug: PathBuf,
    idevicedebugserverproxy: PathBuf,
}

impl IMobileDevice {
    pub fn which() -> Result<Self> {
        Ok(Self {
            idevice_id: which::which(exe!("idevice_id"))?,
            ideviceinfo: which::which(exe!("ideviceinfo"))?,
            ideviceimagemounter: which::which(exe!("ideviceimagemounter"))?,
            ideviceinstaller: which::which(exe!("ideviceinstaller"))?,
            idevicedebug: which::which(exe!("idevicedebug"))?,
            idevicedebugserverproxy: which::which(exe!("idevicedebugserverproxy"))?,
        })
    }

    fn getkey(&self, device: &str, key: &str) -> Result<String> {
        let output = Command::new(&self.ideviceinfo)
            .arg("--udid")
            .arg(device)
            .arg("--key")
            .arg(key)
            .output()?;
        anyhow::ensure!(output.status.success(), "failed to run ideviceinfo");
        Ok(std::str::from_utf8(&output.stdout)?.trim().to_string())
    }

    fn install(&self, device: &str, path: &Path) -> Result<()> {
        let status = Command::new(&self.ideviceinstaller)
            .arg("--udid")
            .arg(device)
            .arg("--install")
            .arg(path)
            .status()?;
        anyhow::ensure!(status.success(), "failed to run ideviceinstaller");
        Ok(())
    }

    fn start(&self, device: &str, bundle_identifier: &str) -> Result<()> {
        let status = Command::new(&self.idevicedebug)
            .arg("--udid")
            .arg(device)
            .arg("run")
            .arg(bundle_identifier)
            .status()?;
        anyhow::ensure!(status.success(), "failed to run idevicedebug");
        Ok(())
    }

    fn disk_image_mounted(&self, device: &str) -> Result<bool> {
        let output = Command::new(&self.ideviceimagemounter)
            .arg("--udid")
            .arg(device)
            .arg("-l")
            .output()?;
        anyhow::ensure!(output.status.success(), "failed to run ideviceimagemounter");
        let num_images: u32 = std::str::from_utf8(&output.stdout)?
            .split_once('[')
            .context("unexpected output")?
            .1
            .split_once(']')
            .context("unexpected output")?
            .0
            .parse()?;
        Ok(num_images > 0)
    }

    pub fn mount_disk_image(&self, env: &BuildEnv, device: &str) -> Result<()> {
        let (major, minor) = self.product_version(device)?;
        let disk_image = env.developer_disk_image(major, minor);
        if self.disk_image_mounted(device)? {
            return Ok(());
        }
        let status = Command::new(&self.ideviceimagemounter)
            .arg("--udid")
            .arg(device)
            .arg(disk_image)
            .status()?;
        anyhow::ensure!(status.success(), "failed to run ideviceimagemounter");
        Ok(())
    }

    pub fn run(&self, env: &BuildEnv, device: &str, path: &Path) -> Result<()> {
        let bundle_identifier = appbundle::app_bundle_identifier(path)?;
        self.mount_disk_image(env, device)?;
        self.install(device, path)?;
        self.start(device, &bundle_identifier)?;
        Ok(())
    }

    pub fn devices(&self, devices: &mut Vec<Device>) -> Result<()> {
        let output = Command::new(&self.idevice_id)
            .arg("-l")
            .arg("-d")
            .output()?;
        anyhow::ensure!(output.status.success(), "failed to run idevice_id");
        let lines = std::str::from_utf8(&output.stdout)?.lines();
        for uuid in lines {
            devices.push(Device {
                backend: Backend::Imd(self.clone()),
                id: uuid.trim().to_string(),
            });
        }
        Ok(())
    }

    pub fn name(&self, device: &str) -> Result<String> {
        self.getkey(device, "DeviceName")
    }

    pub fn platform(&self, _device: &str) -> Result<Platform> {
        Ok(Platform::Ios)
    }

    pub fn arch(&self, device: &str) -> Result<Arch> {
        match self.getkey(device, "CPUArchitecture")?.as_str() {
            "arm64" | "arm64e" => Ok(Arch::Arm64),
            arch => anyhow::bail!("unsupported arch {}", arch),
        }
    }

    pub fn product_version(&self, device: &str) -> Result<(u32, u32)> {
        let version = self.getkey(device, "ProductVersion")?;
        let (major, version) = version.split_once('.').context("invalid product version")?;
        let (minor, _) = version.split_once('.').context("invalid product version")?;
        Ok((major.parse()?, minor.parse()?))
    }

    pub fn details(&self, device: &str) -> Result<String> {
        let name = self.getkey(device, "ProductName")?;
        let version = self.getkey(device, "ProductVersion")?;
        Ok(format!("{} {}", name, version))
    }

    pub fn bundle_path_device(&self, device: &str, bundle_identifier: &str) -> Result<PathBuf> {
        let output = Command::new(&self.ideviceinstaller)
            .arg("--udid")
            .arg(device)
            .arg("-l")
            .arg("-o")
            .arg("xml")
            .output()?;
        anyhow::ensure!(output.status.success(), "failed to run ideviceinstaller");
        let plist: plist::Value = plist::from_reader_xml(&*output.stdout)?;
        let apps = plist.as_array().context("invalid Info.plist")?;
        for app in apps {
            let app = app.as_dictionary().context("invalid Info.plist")?;
            let app_bundle_identifier = app
                .get("CFBundleIdentifier")
                .context("invalid Info.plist")?
                .as_string()
                .context("invalid Info.plist")?;
            if bundle_identifier != app_bundle_identifier {
                continue;
            }
            let path = app
                .get("Path")
                .context("invalid Info.plist")?
                .as_string()
                .context("invalid Info.plist")?;
            return Ok(Path::new(path).to_path_buf());
        }
        anyhow::bail!("app with bundle identifier {} not found", bundle_identifier);
    }

    pub fn start_debug_server_proxy(&self, device: &str, port: u16) -> Result<()> {
        let mut cmd = Command::new(&self.idevicedebugserverproxy);
        cmd.arg("--udid")
            .arg(device)
            .arg("--lldb")
            .arg(port.to_string());
        std::thread::spawn(move || {
            cmd.status().unwrap();
        });
        Ok(())
    }

    pub fn lldb(&self, env: &BuildEnv, device: &str, path: &Path) -> Result<()> {
        let bundle_identifier = appbundle::app_bundle_identifier(path)?;
        self.mount_disk_image(env, device)?;
        self.install(device, path)?;

        let port = 1234;
        let bundle_path = self.bundle_path_device(device, &bundle_identifier)?;
        let work_dir = path.parent().unwrap();
        let script = include_str!("../../scripts/lldb.cmd")
            .replace("{sysroot}", env.ios_sdk().to_str().unwrap())
            .replace("{disk_app}", path.join(env.name()).to_str().unwrap())
            .replace("{device_app}", bundle_path.to_str().unwrap())
            .replace("{device_port}", &port.to_string())
            .replace(
                "{python_file_path}",
                work_dir.join("fruitstrap.py").to_str().unwrap(),
            )
            .replace("{python_command}", "fruitstrap");
        std::fs::write(work_dir.join("fruitstrap.cmd"), script)?;
        std::fs::write(
            work_dir.join("fruitstrap.py"),
            include_str!("../../scripts/lldb.py"),
        )?;
        self.start_debug_server_proxy(device, port)?;
        let status = Command::new("lldb")
            .current_dir(work_dir)
            .arg("-s")
            .arg("fruitstrap.cmd")
            .status()?;
        anyhow::ensure!(status.success(), "failed to run lldb");
        Ok(())
    }
}
