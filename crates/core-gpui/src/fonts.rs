//! The bundled Inter fonts. `font-family.sans` is Inter in every renderer, so core-gpui ships
//! it rather than depending on what the OS has installed.

use std::borrow::Cow;

use gpui::App;

/// Inter 4.1, static weights 400–700 (see `assets/fonts/README.md`).
const INTER: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Inter-Bold.ttf"),
];

pub(crate) fn register(cx: &mut App) {
    cx.text_system()
        .add_fonts(INTER.iter().map(|bytes| Cow::Borrowed(*bytes)).collect())
        .expect("the bundled Inter fonts are valid TTFs");
}
