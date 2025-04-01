use crate::cargo::CrateType;
use crate::devices::Device;
use crate::download::DownloadManager;
use crate::task::TaskRunner;
use crate::{BuildEnv, CompileTarget, Platform};
use anyhow::Result;
use app_store_connect::UnifiedApiKey;
use std::path::Path;

mod build;
mod doctor;
mod new;

pub use build::build;
pub use doctor::doctor;
pub use new::new;

pub fn devices() -> Result<()> {
    for device in Device::list()? {
        println!(
            "{:50}{:20}{:20}{}",
            device.to_string(),
            device.name()?,
            format!("{} {}", device.platform()?, device.arch()?),
            device.details()?,
        );
    }
    Ok(())
}

pub fn run(env: &BuildEnv) -> Result<()> {
    let out = env.executable();
    if let Some(device) = env.target().device() {
        device.run(env, &out)?;
    } else {
        anyhow::bail!("no device specified");
    }
    Ok(())
}

pub fn lldb(env: &BuildEnv) -> Result<()> {
    if let Some(device) = env.target().device() {
        let target = CompileTarget::new(device.platform()?, device.arch()?, env.target().opt());
        let cargo_dir = env
            .build_dir()
            .join(target.opt().to_string())
            .join(target.platform().to_string())
            .join(target.arch().to_string())
            .join("cargo");
        let executable = match target.platform() {
            Platform::Android => env.cargo_artefact(&cargo_dir, target, CrateType::Cdylib)?,
            Platform::Ios => env.output(),
            Platform::Linux => env.output().join(env.name()),
            Platform::Macos => env.executable(),
            Platform::Windows => todo!(),
        };
        let lldb_server = match target.platform() {
            Platform::Android => Some(env.lldb_server(target)?),
            _ => None,
        };
        device.lldb(env, &executable, lldb_server.as_deref())?;
    } else {
        anyhow::bail!("no device specified");
    }
    Ok(())
}

pub fn create_apple_api_key(
    issuer_id: &str,
    key_id: &str,
    private_key: &Path,
    api_key: &Path,
) -> Result<()> {
    UnifiedApiKey::from_ecdsa_pem_path(issuer_id, key_id, private_key)?.write_json_file(api_key)?;
    Ok(())
}

pub fn test(env: &BuildEnv) -> Result<()> {
    let platform_dir = env.platform_dir();
    std::fs::create_dir_all(&platform_dir)?;

    let mut runner = TaskRunner::new(3, env.verbose());

    runner.start_task("Fetch precompiled artifacts");
    let manager = DownloadManager::new(env)?;
    if !env.offline() {
        manager.prefetch()?;
        runner.end_verbose_task();
    }

    runner.start_task(format!("Build test `{}`", env.name));
    let bin_target = env.target().platform() != Platform::Android;
    let has_lib = env.root_dir().join("src").join("lib.rs").exists();

    if bin_target || has_lib {
        if env.target().platform() == Platform::Android && env.config().android().gradle {
            crate::gradle::prepare(env)?;
        }
        for target in env.target().compile_targets() {
            let arch_dir = platform_dir.join(target.arch().to_string());
            let mut cargo = env.cargo_test(target, &arch_dir.join("cargo"))?;
            if !bin_target {
                cargo.arg("--lib");
            }
            let executable_unittests = cargo.build_executable_unittests()?;

            runner.start_task(format!("Run test {}", env.target().format()));

            if let Some(device) = env.target().device() {
                device.test(env, &executable_unittests)?;
            } else {
                anyhow::bail!("no device specified");
            }
        }
        runner.end_verbose_task();
    }

    Ok(())
}
