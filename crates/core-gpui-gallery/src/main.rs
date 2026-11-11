//! `brrritscold-core-gpui-gallery`: with no arguments, opens the gallery window;
//! `capture --out <dir>` writes the headless captures instead.

use std::path::Path;

use anyhow::bail;
use brrritscold_core_gpui::Root;
use brrritscold_core_gpui_gallery::{capture::capture, gallery::Gallery};
use gpui::{App, AppContext as _, Bounds, WindowBounds, WindowOptions, px, size};

const USAGE: &str = "usage: brrritscold-core-gpui-gallery [capture --out <dir>]";

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => {
            open_window();
            Ok(())
        }
        ["capture", "--out", out] => capture(Path::new(out)),
        _ => bail!(USAGE),
    }
}

fn open_window() {
    gpui_platform::application().run(|cx: &mut App| {
        brrritscold_core_gpui::init(cx);
        let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let gallery = cx.new(|_| Gallery::new());
                cx.new(|cx| Root::new(gallery, window, cx))
            },
        )
        .expect("the gallery window opens");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
