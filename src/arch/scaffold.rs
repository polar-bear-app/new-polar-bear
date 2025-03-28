use crate::utils::{
    application_context::get_application_context, config, logging::PolarBearExpectation,
};
use std::fs;
use tar::Archive;
use winit::platform::android::activity::AndroidApp;
use xz2::read::XzDecoder;

use super::process::Log;

pub fn scaffold(android_app: AndroidApp, log: Log) {
    let context = get_application_context().pb_expect("Failed to get application context");
    println!("Application context: {:?}", context);
    let fs_root = std::path::Path::new(config::ARCH_FS_ROOT);
    let tar_file = android_app
        .asset_manager()
        .open(
            std::ffi::CString::new(config::ARCH_FS_ARCHIVE)
                .pb_expect("Failed to create CString from ARCH_FS_ARCHIVE")
                .as_c_str(),
        )
        .pb_expect("Failed to open Arch Linux FS .tar.xz in asset manager");

    let mut should_pacstrap = false;
    if !fs_root.exists()
        || fs::read_dir(fs_root)
            .pb_expect("Failed to read fs_root directory")
            .next()
            .is_none()
    {
        should_pacstrap = true;
        log("Arch Linux is not installed! Installing...".to_string());
    }

    if should_pacstrap {
        log("(This may take a few minutes.)".to_string());

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
    }
}
