use crate::{
    arch::run::{arch_run, arch_run_with_log},
    utils::{
        config,
        logging::{log_format, log_to_panel, PolarBearExpectation},
    },
    wayland::compositor::PolarBearCompositor,
};
use eframe::{egui, NativeOptions};
use std::{
    collections::VecDeque,
    panic,
    sync::{Arc, Mutex},
    thread,
};

pub struct PolarBearApp {
    logs: Arc<Mutex<VecDeque<String>>>,
    compositor: Arc<Mutex<Option<PolarBearCompositor>>>,
}

#[cfg(target_os = "android")]
use crate::{arch::scaffold, utils::application_context::ApplicationContext};

impl PolarBearApp {
    pub fn run(options: NativeOptions) -> Result<(), eframe::Error> {
        let logs = Arc::new(Mutex::new(VecDeque::new()));
        #[cfg(target_os = "android")]
        let cloned_android_app = options
            .android_app
            .clone()
            .pb_expect("Failed to clone AndroidApp");
        #[cfg(target_os = "android")]
        ApplicationContext::build(&cloned_android_app);

        let compositor = Arc::new(Mutex::new(None));
        let app: PolarBearApp = PolarBearApp {
            logs: Arc::clone(&logs),
            compositor: Arc::clone(&compositor),
        };
        thread::spawn(move || {
            let result = panic::catch_unwind(|| {
                // Step 1. Setup Arch FS if not already installed
                #[cfg(target_os = "android")]
                scaffold::scaffold(&cloned_android_app, &logs);

                // Step 2. Install dependencies if not already installed
                arch_run_with_log(&["uname", "-a"], &logs);
                loop {
                    let installed = arch_run(&["pacman", "-Qg", "plasma"])
                        .wait()
                        .pb_expect("pacman -Qg plasma failed")
                        .success();
                    if installed {
                        match PolarBearCompositor::build() {
                            Ok(c) => {
                                let mut comp =
                                    compositor.lock().pb_expect("Failed to lock compositor");
                                *comp = Some(c);

                                log_to_panel(
                                    &log_format(
                                        "POLAR BEAR COMPOSITOR STARTED",
                                        "Polar Bear Compositor started successfully",
                                    ),
                                    &logs,
                                );

                                arch_run_with_log(
                                    &[
                                        "sh",
                                        "-c",
                                        &format!(
                                            "HOME=/root XDG_RUNTIME_DIR={} WAYLAND_DISPLAY={} WAYLAND_DEBUG=client weston",
                                            config::XDG_RUNTIME_DIR,
                                            config::WAYLAND_SOCKET_NAME
                                        ),
                                    ],
                                    &logs,
                                );
                            }
                            Err(e) => {
                                log_to_panel(
                                    &log_format(
                                        "POLAR BEAR COMPOSITOR RUNTIME ERROR",
                                        &format!("{}", e),
                                    ),
                                    &logs,
                                );
                            }
                        }
                        break;
                    } else {
                        arch_run(&["rm", "/var/lib/pacman/db.lck"]);
                        arch_run_with_log(
                            &["pacman", "-Syu", "plasma", "weston", "--noconfirm"],
                            &logs,
                        );
                    }
                }
            });
            if let Err(e) = result {
                let error_msg = e
                    .downcast_ref::<&str>()
                    .map(|s| *s)
                    .or_else(|| e.downcast_ref::<String>().map(|s| s.as_str()))
                    .unwrap_or("Unknown error");

                log_to_panel(error_msg, &logs);
            }
        });
        eframe::run_native("Polar Bear", options, Box::new(|_cc| Ok(Box::new(app))))
    }
}

impl eframe::App for PolarBearApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("log_panel")
            .resizable(true)
            .default_width(320.0)
            .width_range(80.0..=ctx.available_rect().width() / 2.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Logs");
                });
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let logs = self.logs.lock().pb_expect("Failed to lock logs");
                        ui.label(logs.iter().cloned().collect::<Vec<_>>().join("\n"))
                    });
            });
    }
}
