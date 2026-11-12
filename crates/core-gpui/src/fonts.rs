//! The bundled fonts: Inter for `font-family.sans` and JetBrains Mono for `font-family.mono`.
//! Both head their stack in every renderer, so core-gpui ships them rather than depending on
//! what the OS has installed.

use std::borrow::Cow;

use gpui::App;

/// Inter 4.1, static weights 400–700 (see `assets/fonts/README.md`).
const INTER: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Inter-Bold.ttf"),
];

/// JetBrains Mono 2.304, static weights 400 and 700 (see `assets/fonts/README.md`).
const JETBRAINS_MONO: [&[u8]; 2] = [
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
];

pub(crate) fn register(cx: &mut App) {
    cx.text_system()
        .add_fonts(
            INTER
                .iter()
                .chain(&JETBRAINS_MONO)
                .map(|bytes| Cow::Borrowed(*bytes))
                .collect(),
        )
        .expect("the bundled Inter and JetBrains Mono fonts are valid TTFs");
}
