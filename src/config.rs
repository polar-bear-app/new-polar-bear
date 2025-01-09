#[cfg(target_os = "android")]
pub const ARCH_FS_ROOT: &str = "/data/data/app.polarbear/files/archlinux";
#[cfg(target_os = "macos")]
pub const ARCH_FS_ROOT: &str = "~/OrbStack/arch";

pub const ARCH_FS_ARCHIVE: &str = "archlinux-aarch64-pd-v4.6.0.tar.xz";

pub const WAYLAND_SOCKET_NAME: &str = "wayland-pb";

pub const MAX_PANEL_LOG_ENTRIES: usize = 100;
