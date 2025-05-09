use super::process::ArchProcess;
use crate::{
    app::build::{PolarBearBackend, WaylandBackend, WebviewBackend},
    utils::{application_context::get_application_context, config, logging::PolarBearExpectation},
    wayland::compositor::Compositor,
};
use smithay::utils::Clock;
use std::{
    fs,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};
use tar::Archive;
use winit::platform::android::activity::AndroidApp;
use xz2::read::XzDecoder;

pub struct SetupOptions {
    pub install_packages: String,
    pub checking_command: String,
    pub username: String,
    pub android_app: AndroidApp,
    pub mpsc_sender: Sender<String>,
}

/// Setup is a process that should be done **only once** when the user installed the app.
/// The setup process consists of several stages.
/// Each stage is a function that takes the `SetupOptions` and returns a `StageOutput`.
type SetupStage = Box<dyn Fn(&SetupOptions) -> StageOutput + Send>;

/// Each stage should indicate whether the associated task is done previously or not.
/// Thus, it should return a finished status if the task is done, so that the setup process can move on to the next stage.
/// Otherwise, it should return a `JoinHandle`, so that the setup process can wait for the task to finish, but not block the main thread so that the setup progress can be reported to the user.
type StageOutput = Option<JoinHandle<()>>;

fn setup_arch_fs(options: &SetupOptions) -> StageOutput {
    let context = get_application_context().pb_expect("Failed to get application context");
    println!("Application context: {:?}", context);
    let fs_root = std::path::Path::new(config::ARCH_FS_ROOT);

    if !fs_root.exists()
        || fs::read_dir(fs_root)
            .pb_expect("Failed to read fs_root directory")
            .next()
            .is_none()
    {
        let android_app = options.android_app.clone();
        let mpsc_sender = options.mpsc_sender.clone();
        return Some(thread::spawn(move || {
            mpsc_sender
                .send("Extracting Arch Linux FS...".to_string())
                .pb_expect("Failed to send log message");

            let tar_file = android_app
                .asset_manager()
                .open(
                    std::ffi::CString::new(config::ARCH_FS_ARCHIVE)
                        .pb_expect("Failed to create CString from ARCH_FS_ARCHIVE")
                        .as_c_str(),
                )
                .pb_expect("Failed to open Arch Linux FS .tar.xz in asset manager");

            // Ensure the extracted directory is clean
            let extracted_dir = &context.data_dir.join("archlinux-aarch64");
            fs::remove_dir_all(extracted_dir).unwrap_or(());

            // Extract tar file directly to the final destination
            let tar = XzDecoder::new(tar_file);
            let mut archive = Archive::new(tar);
            archive
                .unpack(context.data_dir.clone())
                .pb_expect("Failed to extract Arch Linux FS .tar.xz file");

            // Move the extracted files to the final destination
            fs::rename(extracted_dir, fs_root)
                .pb_expect("Failed to rename extracted files to final destination");
        }));
    } else {
        return None;
    }
}

fn fix_tmp_permissions(_options: &SetupOptions) -> StageOutput {
    // Fix "/tmp" can be written by others
    ArchProcess::exec("chmod 700 /tmp")
        .wait()
        .pb_expect("chmod 700 /tmp failed");
    None
}

fn create_normal_user(options: &SetupOptions) -> StageOutput {
    let username = options.username.clone();
    if !ArchProcess::exec(&format!("id {username}"))
        .wait_with_output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        let command = format!("useradd -m -G wheel {username} && passwd -d {username}");
        ArchProcess::exec(&command)
            .wait()
            .pb_expect(&format!("{} failed", command));
    }

    None
}

fn install_dependencies(options: &SetupOptions) -> StageOutput {
    let SetupOptions {
        install_packages,
        checking_command,
        mpsc_sender,
        username: _,
        android_app: _,
    } = options;

    let checking_command = checking_command.clone();
    let installed = move || {
        ArchProcess::exec(&checking_command)
            .wait()
            .pb_expect("Failed to check whether the installation target is installed")
            .success()
    };

    if installed() {
        return None;
    }

    let install_packages = install_packages.clone();
    let mpsc_sender = mpsc_sender.clone();
    return Some(thread::spawn(move || {
        loop {
            ArchProcess::exec("rm -f /var/lib/pacman/db.lck"); // Install dependencies
            ArchProcess::exec(&format!(
                "stdbuf -oL pacman -Syu {} --noconfirm --noprogressbar",
                install_packages
            ))
            .with_log(|it| {
                mpsc_sender.send(it).pb_expect("Failed to send log message");
            });
            if installed() {
                break;
            }
        }
    }));
}

pub fn setup(android_app: AndroidApp) -> PolarBearBackend {
    let (sender, receiver) = mpsc::channel();
    let progress = Arc::new(Mutex::new(0));

    let options = SetupOptions {
        username: "polarbear".to_string(),
        install_packages: config::PACMAN_INSTALL_PACKAGES.to_string(),
        checking_command: config::PACMAN_CHECKING_COMMAND.to_string(),
        android_app,
        mpsc_sender: sender.clone(),
    };

    let stages: Vec<SetupStage> = vec![
        Box::new(setup_arch_fs),        // Step 1. Setup Arch FS
        Box::new(create_normal_user),   // Step 2. Create normal user
        Box::new(install_dependencies), // Step 3. Install dependencies
    ];

    let fully_installed = 'outer: loop {
        for (i, stage) in stages.iter().enumerate() {
            if let Some(handle) = stage(&options) {
                let progress_clone = progress.clone();
                thread::spawn(move || {
                    let progress = progress_clone;
                    let progress_value = ((i) as u16 * 100 / stages.len() as u16) as u16;
                    *progress.lock().unwrap() = progress_value;
                    // Wait for the current stage to finish
                    handle.join().pb_expect("Failed to join thread");

                    // Process the remaining stages in the same loop
                    for (j, next_stage) in stages.iter().enumerate().skip(i + 1) {
                        let progress_value = ((j) as u16 * 100 / stages.len() as u16) as u16;
                        *progress.lock().unwrap() = progress_value;
                        if let Some(next_handle) = next_stage(&options) {
                            next_handle.join().pb_expect("Failed to join thread");

                            // Increment progress and send it
                            let next_progress_value =
                                ((j + 1) as u16 * 100 / stages.len() as u16) as u16;
                            *progress.lock().unwrap() = next_progress_value;
                        }
                    }

                    // All stages are done, we need to replace the WebviewBackend with the WaylandBackend
                    // Or, easier, just restart the whole app
                    *progress.lock().unwrap() = 100;
                    sender
                        .send("Installation finished, please restart the app".to_string())
                        .pb_expect("Failed to send installation finished message");
                });

                // Setup is still running in the background, but we need to return control
                // so that the main thread can continue to report progress to the user
                break 'outer false;
            }
        }

        // All stages were done previously, no need to wait for anything
        break 'outer true;
    };

    if fully_installed {
        PolarBearBackend::Wayland(WaylandBackend {
            compositor: Compositor::build().pb_expect("Failed to build compositor"),
            graphic_renderer: None,
            clock: Clock::new(),
            key_counter: 0,
            scale_factor: 1.0,
        })
    } else {
        PolarBearBackend::WebView(WebviewBackend::build(receiver, progress))
    }
}
