use crate::{
    app::{compositor::PolarBearCompositor, renderer::PolarBearRenderer},
    arch::run::{arch_run, arch_run_with_log},
    utils::{
        config,
        logging::{log_format, PolarBearExpectation},
        patches::to_scan_code,
    },
};
use eframe::{egui, NativeOptions};
use egui_winit::winit::platform::android::activity::WindowManagerFlags;
use smithay::{
    backend::input::{
        KeyState::{Pressed, Released},
        TouchSlot,
    },
    input::{
        keyboard::FilterResult,
        touch::{DownEvent, MotionEvent, UpEvent},
    },
    utils::{Serial, SERIAL_COUNTER},
};
use std::{
    collections::VecDeque,
    panic,
    sync::{Arc, Mutex},
    thread,
};

pub struct Shared {
    pub compositor: Option<PolarBearCompositor>,
    ctx: Option<egui::Context>,
    logs: VecDeque<String>,
}

impl Shared {
    pub fn log(&mut self, content: String) {
        self.logs.push_back(content);
        // Ensure the logs size stays at most 20
        if self.logs.len() > config::MAX_PANEL_LOG_ENTRIES {
            self.logs.pop_front();
        }
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
}

pub struct PolarBearApp {
    shared: Arc<Mutex<Shared>>,
}

use crate::{arch::scaffold, utils::application_context::ApplicationContext};

impl PolarBearApp {
    pub fn run(options: NativeOptions) -> Result<(), eframe::Error> {
        let cloned_android_app = options
            .android_app
            .clone()
            .pb_expect("Failed to clone AndroidApp");
        ApplicationContext::build(&cloned_android_app);

        // Enable fullscreen immersive mode
        cloned_android_app.set_window_flags(
            WindowManagerFlags::FULLSCREEN | WindowManagerFlags::LAYOUT_IN_SCREEN,
            WindowManagerFlags::empty(),
        );

        let shared = Arc::new(Mutex::new(Shared {
            compositor: None,
            ctx: None,
            logs: VecDeque::new(),
        }));

        let app = PolarBearApp {
            shared: Arc::clone(&shared),
        };

        thread::spawn(move || {
            let result = panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let log = |it| {
                    shared.lock().unwrap().log(it);
                };

                // Step 1. Setup Arch FS if not already installed
                scaffold::scaffold(&cloned_android_app, log);

                // Step 2. Install dependencies if not already installed
                arch_run_with_log("uname -a", log);
                loop {
                    let installed = arch_run(&"pacman -Qg plasma")
                        .wait()
                        .pb_expect("pacman -Qg plasma failed")
                        .success();
                    if installed {
                        match PolarBearCompositor::build(&cloned_android_app) {
                            Ok(compositor) => {
                                {
                                    shared.lock().unwrap().compositor.replace(compositor);
                                }
                                arch_run_with_log(
                                    &format!(
                                            // "HOME=/root XDG_RUNTIME_DIR={} WAYLAND_DISPLAY={} WAYLAND_DEBUG=client weston --fullscreen 2>&1",
                                            "HOME=/root XDG_RUNTIME_DIR={} WAYLAND_DISPLAY={} WAYLAND_DEBUG=client dbus-launch startplasma-wayland 2>&1",
                                            config::XDG_RUNTIME_DIR,
                                            config::WAYLAND_SOCKET_NAME),
                                    log,
                                );
                            }
                            Err(e) => {
                                log(log_format(
                                    "POLAR BEAR COMPOSITOR RUNTIME ERROR",
                                    &format!("{}", e),
                                ));
                            }
                        }
                        break;
                    } else {
                        arch_run("rm /var/lib/pacman/db.lck");
                        arch_run_with_log("pacman -Syu plasma weston --noconfirm", log);
                    }
                }
            }));
            if let Err(e) = result {
                let error_msg = e
                    .downcast_ref::<&str>()
                    .map(|s| *s)
                    .or_else(|| e.downcast_ref::<String>().map(|s| s.as_str()))
                    .unwrap_or("Unknown error");

                shared.lock().unwrap().log(log_format(
                    "POLAR BEAR COMPOSITOR RUNTIME ERROR",
                    &format!("{}", error_msg),
                ));
            }
        });
        eframe::run_native("Polar Bear", options, Box::new(|_cc| Ok(Box::new(app))))
    }
}

impl eframe::App for PolarBearApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        for event in &raw_input.events {
            match event {
                egui::Event::Key {
                    key,
                    physical_key,
                    pressed,
                    repeat,
                    modifiers,
                } => {
                    if let Some(key_code) = physical_key {
                        if let Some(scan_code) = to_scan_code(*key_code) {
                            let mut shared = self.shared.lock().unwrap();
                            let compositor = shared.compositor.as_mut().unwrap();
                            let surface = compositor.get_surface();
                            let key_state = if *pressed { Pressed } else { Released };
                            let keyboard = &compositor.keyboard;
                            keyboard.set_focus(&mut compositor.state, surface.clone(), 0.into());
                            keyboard.input::<(), _>(
                                &mut compositor.state,
                                (scan_code + 8).into(),
                                key_state,
                                0.into(),
                                0,
                                |_, _, _| FilterResult::Forward,
                            );
                        }
                    }
                }
                egui::Event::Touch {
                    device_id,
                    id,
                    pos,
                    phase,
                    ..
                } => {
                    let mut shared = self.shared.lock().unwrap();
                    let compositor = shared.compositor.as_mut().unwrap();
                    let surface = compositor.get_surface();
                    let touch = &compositor.touch;
                    let slot = Option::Some(id.0 as u32 + 1).into();
                    let scale_factor = _ctx.native_pixels_per_point().unwrap_or(1.0);
                    let location =
                        ((pos.x * scale_factor) as f64, (pos.y * scale_factor) as f64).into();
                    let serial = SERIAL_COUNTER.next_serial();
                    let time = compositor.start_time.elapsed().as_millis() as u32;

                    match phase {
                        egui::TouchPhase::Start => {
                            touch.down(
                                &mut compositor.state,
                                surface
                                    .clone()
                                    .map(|surface| (surface, (0f64, 0f64).into())),
                                &DownEvent {
                                    slot,
                                    location,
                                    serial,
                                    time,
                                },
                            );
                        }
                        egui::TouchPhase::Move => {
                            touch.motion(
                                &mut compositor.state,
                                surface
                                    .clone()
                                    .map(|surface| (surface, (0f64, 0f64).into())),
                                &MotionEvent {
                                    slot,
                                    location,
                                    time,
                                },
                            );
                        }
                        egui::TouchPhase::End => {
                            touch.up(&mut compositor.state, &UpEvent { slot, serial, time });
                        }
                        egui::TouchPhase::Cancel => {
                            touch.cancel(&mut compositor.state);
                        }
                    }

                    touch.frame(&mut compositor.state);
                }
                _ => {}
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if cfg!(debug_assertions) {
            ctx.set_debug_on_hover(true);
        }

        if cfg!(debug_assertions) {
            egui::Window::new("Inspection")
                .resizable(true)
                .default_width(320.0)
                .show(ctx, |ui| {
                    ctx.inspection_ui(ui);
                });
        }

        egui::Window::new("Logs")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.label(
                            self.shared
                                .lock()
                                .unwrap()
                                .logs
                                .iter()
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Vec2::new(0.0, 0.0)))
            .show(ctx, |ui| {
                let renderer = PolarBearRenderer {
                    painter: ui.painter().clone(),
                };
                if let Some(compositor) = self.shared.lock().unwrap().compositor.as_mut() {
                    match compositor.draw(renderer, ui.available_size()) {
                        Ok(_) => {}
                        Err(e) => {
                            self.shared.lock().unwrap().log(log_format(
                                "POLAR BEAR COMPOSITOR DRAW ERROR",
                                &format!("{}", e),
                            ));
                        }
                    }
                };
            });

        self.shared.lock().unwrap().ctx = Some(ctx.clone());
    }
}
