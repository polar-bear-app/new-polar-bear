pub mod app {
    pub mod eframe;
    pub mod run;
}
pub mod arch {
    pub mod run;

    #[cfg(target_os = "android")]
    pub mod scaffold;
}
pub mod wayland {
    pub mod compositor;
    pub mod minimal;
}
pub mod utils {
    pub mod config;
    pub mod logging;

    #[cfg(target_os = "android")]
    pub mod application_context;
}

#[cfg(target_os = "android")]
pub mod android_main;
