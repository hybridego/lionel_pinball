mod app;
mod game;
mod ui;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

fn main() -> eframe::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Log to stdout (if you run with `RUST_LOG=debug`)
        // env_logger::init(); // we didn't add env_logger, skipping for now

        let native_options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([800.0, 900.0])
                .with_min_inner_size([300.0, 400.0]),

            ..Default::default()
        };
        eframe::run_native(
            "Pinball Gacha",
            native_options,
            Box::new(|cc| Ok(Box::new(app::PinballApp::new(cc)))),
        )
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Redirect `log` message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");

            let canvas = document
                .get_element_by_id("the_canvas_id")
                .expect("Failed to find the_canvas_id")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("the_canvas_id was not a HtmlCanvasElement");

            eframe::WebRunner::new()
                .start(
                    canvas,
                    web_options,
                    Box::new(|cc| Ok(Box::new(app::PinballApp::new(cc)))),
                )
                .await
                .expect("failed to start eframe");
        });
        Ok(())
    }
}
