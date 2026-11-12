//! Headless captures: one PNG per registry variant × theme, at the test window's scale
//! (2x), cropped to the component plus a small margin.
//!
//! Captures are local only: never committed, never compared by a program. A person opens
//! them, alone or in `cargo xtask captures compare`. They need a Vulkan driver (lavapipe
//! works) and no display server.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{Context as _, bail, ensure};
use brrritscold_core_gpui::{Mode, Root, Theme, ThemePreference};
use brrritscold_core_gpui_tokens::primitive::{font_family, font_weight};
use gpui::{
    AppContext as _, Bounds, Context, Font, FontWeight, HeadlessAppContext, IntoElement,
    ParentElement as _, Pixels, Render, Rgba, Size, Styled as _, TextSystem, Window, div, font, px,
    size,
};
use serde_json::json;

use crate::registry::{Variant, registry};

/// The capture window, in logical pixels. A board that outgrows it fails the run.
const WINDOW: Size<Pixels> = size(px(1280.), px(960.));

/// Logical pixels kept around each component, so its shadow and edges show.
const MARGIN: f32 = 4.;

/// Captures every registry variant in both themes into `out`, plus `index.json` listing
/// each file with its component, props, theme, Storybook story and the theme's
/// `background` token (what a comparison flattens a transparent export onto).
pub fn capture(out: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    let mut index = Vec::new();
    for mode in [Mode::Light, Mode::Dark] {
        let mut cx = HeadlessAppContext::with_platform(
            gpui_platform::current_platform(true).text_system(),
            Arc::new(()),
            gpui_platform::current_headless_renderer,
        );
        cx.update(|cx| {
            brrritscold_core_gpui::init(cx);
            Theme::set_preference(mode.pick(ThemePreference::Light, ThemePreference::Dark), cx);
        });
        cx.update(|cx| ensure_bundled_fonts(cx.text_system()))?;
        let theme = mode.pick("light", "dark");
        let background = hex(mode.semantic().background.to_rgb());
        for entry in registry() {
            let variants = Rc::new(entry.variants());
            let shots = shoot(&mut cx, variants.clone())
                .with_context(|| format!("capturing {} ({theme})", entry.name))?;
            for (variant, image) in variants.iter().zip(shots) {
                let file = format!(
                    "{}--{}--{theme}.png",
                    entry.name.to_lowercase(),
                    variant.slug()
                );
                image
                    .save(out.join(&file))
                    .with_context(|| format!("writing {file}"))?;
                let props: serde_json::Map<String, serde_json::Value> = variant
                    .props
                    .iter()
                    .map(|(property, value)| ((*property).to_owned(), json!(value)))
                    .collect();
                index.push(json!({
                    "component": entry.name,
                    "props": props,
                    "theme": theme,
                    "file": file,
                    "story": variant.story,
                    "background": background,
                }));
            }
        }
    }
    let index = serde_json::to_string_pretty(&json!({ "captures": index }))?;
    std::fs::write(out.join("index.json"), index).context("writing index.json")?;
    Ok(())
}

/// `#rrggbb`.
fn hex(color: Rgba) -> String {
    let channel = |c: f32| (c * 255.).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

/// Fails unless each bundled family resolves to itself with a face per weight the library
/// uses: sans to Inter at 400 and 500, mono to JetBrains Mono at 400 and 700. A capture then
/// never shows a fallback font that looks almost right.
fn ensure_bundled_fonts(text_system: &TextSystem) -> anyhow::Result<()> {
    ensure_family(
        text_system,
        font_family::SANS.family,
        font_weight::NORMAL,
        font_weight::MEDIUM,
    )?;
    ensure_family(
        text_system,
        font_family::MONO.family,
        font_weight::NORMAL,
        font_weight::BOLD,
    )
}

/// Fails unless `family` resolves to itself at both weights, with a different face for each.
fn ensure_family(
    text_system: &TextSystem,
    family: &str,
    regular: FontWeight,
    heavier: FontWeight,
) -> anyhow::Result<()> {
    let resolve = |weight| {
        let id = text_system.resolve_font(&Font {
            weight,
            ..font(family.to_owned())
        });
        let resolved = text_system.get_font_for_id(id).map(|f| f.family);
        (id, resolved)
    };
    let (regular_id, regular_family) = resolve(regular);
    let (heavier_id, heavier_family) = resolve(heavier);
    for (weight, resolved) in [(regular, &regular_family), (heavier, &heavier_family)] {
        ensure!(
            resolved.as_deref() == Some(family),
            "`{family}` at weight {} resolves to {resolved:?}; core_gpui::init should have \
             registered it",
            weight.0
        );
    }
    ensure!(
        regular_id != heavier_id,
        "`{family}` has no face at weight {}; it would render as {}",
        heavier.0,
        regular.0
    );
    Ok(())
}

/// Renders every variant on one board and crops each out of a single screenshot.
fn shoot(
    cx: &mut HeadlessAppContext,
    variants: Rc<Vec<Variant>>,
) -> anyhow::Result<Vec<image::RgbaImage>> {
    let bounds = Rc::new(RefCell::new(vec![None; variants.len()]));
    let window = cx.open_window(WINDOW, |window, cx| {
        let board = cx.new(|_| Board {
            variants,
            bounds: bounds.clone(),
        });
        cx.new(|cx| Root::new(board, window, cx))
    })?;
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))?;
    // Icons rasterise off the first frame; let that land, then draw the settled frame.
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx)
    })?;
    let screenshot = cx.capture_screenshot(window.into())?;
    cx.update_window(window.into(), |_, window, _| window.remove_window())?;

    let scale = screenshot.width() as f32 / f32::from(WINDOW.width);
    let bounds = bounds.borrow();
    bounds
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let Some(cell) = cell else {
                bail!("variant {index} was never laid out");
            };
            crop(&screenshot, *cell, scale)
        })
        .collect()
}

fn crop(
    screenshot: &image::RgbaImage,
    cell: Bounds<Pixels>,
    scale: f32,
) -> anyhow::Result<image::RgbaImage> {
    let device = |logical: Pixels, margin: f32| ((f32::from(logical) + margin) * scale).round();
    let left = device(cell.origin.x, -MARGIN);
    let top = device(cell.origin.y, -MARGIN);
    let right = device(cell.origin.x + cell.size.width, MARGIN);
    let bottom = device(cell.origin.y + cell.size.height, MARGIN);
    ensure!(left >= 0. && top >= 0., "a cell starts outside the window");
    ensure!(
        right <= screenshot.width() as f32 && bottom <= screenshot.height() as f32,
        "the board outgrew the {}x{} capture window; enlarge WINDOW",
        f32::from(WINDOW.width),
        f32::from(WINDOW.height)
    );
    let (x, y) = (left as u32, top as u32);
    let (width, height) = ((right - left) as u32, (bottom - top) as u32);
    Ok(image::imageops::crop_imm(screenshot, x, y, width, height).to_image())
}

/// Every variant, wrapped so its laid-out bounds are recorded.
struct Board {
    variants: Rc<Vec<Variant>>,
    bounds: Rc<RefCell<Vec<Option<Bounds<Pixels>>>>>,
}

impl Render for Board {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_4()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_4()
            .children(self.variants.iter().enumerate().map(|(index, variant)| {
                let bounds = self.bounds.clone();
                div()
                    .flex()
                    .child(variant.render(("capture", index)))
                    .on_children_prepainted(move |children, _, _| {
                        bounds.borrow_mut()[index] = children.first().copied();
                    })
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod hex {
        use super::*;

        #[test]
        fn should_write_the_background_tokens_as_css_hex() {
            assert_eq!(hex(Mode::Light.semantic().background.to_rgb()), "#ffffff");
            assert_eq!(hex(Mode::Dark.semantic().background.to_rgb()), "#0a0a0a");
        }
    }
}
