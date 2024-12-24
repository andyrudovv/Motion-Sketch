use skia_safe::{gpu, Color, Paint, PaintStyle, Surface, Canvas};
use raw_window_handle::{HasWindowHandle};
use slint::platform::WindowAdapter;

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    // Initialize your Slint UI
    let ui = AppWindow::new()?;
    let window = ui.window();
    let window_handle = window.window_handle();

    // Access the raw window handle
    let raw_window_handle = window_handle.window_handle().unwrap();

    // Set up Vulkan context for Skia

    ui.run()?;
    Ok(())
}
