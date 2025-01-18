use eframe::{egui::ViewportBuilder, EventLoopBuilder, EventLoopBuilderHook, NativeOptions};
use polar_bear::app::eframe::PolarBearApp;

fn main() -> Result<(), eframe::Error> {
    // Define the hook function
    let event_loop_hook: EventLoopBuilderHook = Box::new(|builder: &mut EventLoopBuilder<_>| {
        // Modify the builder as needed
        builder = None;
    });

    let options = NativeOptions {
        viewport: ViewportBuilder {
            fullscreen: Some(true),
            ..Default::default()
        },
        event_loop_builder: Some(event_loop_hook), // Assign the hook function
        ..Default::default()
    };

    PolarBearApp::run(options)
}
