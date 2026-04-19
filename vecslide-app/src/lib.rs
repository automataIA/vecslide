use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};
use vecslide_core::theme::ThemeColors;

mod components;
pub mod export;
pub mod import;
mod pages;
pub mod theme_extract;
pub mod tts;
pub mod typst_world;

/// Prefix for static assets served from `vecslide-app/public/`.
/// Empty in local/dev, `/vecslide` on GitHub Pages (matches `Trunk.ghpages.toml`
/// `public_url = "/vecslide/"` and the Router `base`).
pub const ASSET_BASE: &str = if cfg!(feature = "gh-pages") { "/vecslide" } else { "" };

use crate::pages::editor::Editor;
use crate::pages::home::Home;
use crate::pages::not_found::NotFound;

/// Global dark-mode toggle: `false` = bumblebee (light), `true` = business (dark).
/// Stored in context so all routes can read/write it without remounting.
#[derive(Clone, Copy)]
pub struct DarkMode(pub RwSignal<bool>);

/// Global loaded-file context: bytes of the `.vecslide` file selected in Home.
/// Home writes → navigate → Editor reads on mount and clears.
#[derive(Clone, Copy)]
pub struct LoadedFile(pub RwSignal<Option<Vec<u8>>>);

/// Reactive DaisyUI theme colors (hex), updated when dark mode toggles.
#[derive(Clone, Copy)]
pub struct ThemeColorsCtx(pub RwSignal<ThemeColors>);

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let dark = RwSignal::new(false);
    provide_context(DarkMode(dark));

    let loaded_file: RwSignal<Option<Vec<u8>>> = RwSignal::new(None);
    provide_context(LoadedFile(loaded_file));

    let theme_colors = RwSignal::new(ThemeColors::default());
    provide_context(ThemeColorsCtx(theme_colors));

    // Re-extract theme colors from the DOM whenever dark mode changes.
    // Uses request_animation_frame to ensure the browser has applied the new data-theme.
    Effect::new(move |_| {
        let _ = dark.get(); // subscribe to dark mode changes
        // Defer extraction to the next frame so the browser applies the new data-theme first.
        request_animation_frame(move || {
            theme_colors.set(theme_extract::extract_theme_colors());
        });
    });

    view! {
        <Html
            attr:lang="en"
            attr:dir="ltr"
            attr:data-theme=move || if dark.get() { "business" } else { "bumblebee" }
        />
        <Title text="VecSlide" />
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router base=if cfg!(feature = "gh-pages") { "/vecslide" } else { "" }>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=Home />
                <Route path=path!("/editor") view=Editor />
            </Routes>
        </Router>
    }
}
