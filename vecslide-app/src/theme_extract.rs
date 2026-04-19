//! Extract DaisyUI theme colors from the live DOM via `getComputedStyle`.

use vecslide_core::theme::{oklch_to_hex, ThemeColors};
use web_sys::window;

/// CSS variable names for the 19 DaisyUI semantic colors (without `--color-` prefix).
const COLOR_NAMES: &[&str] = &[
    "primary",
    "primary-content",
    "secondary",
    "secondary-content",
    "accent",
    "accent-content",
    "neutral",
    "neutral-content",
    "base-100",
    "base-200",
    "base-300",
    "base-content",
    "info",
    "info-content",
    "success",
    "success-content",
    "warning",
    "warning-content",
    "error",
    "error-content",
];

/// Read the current DaisyUI theme colors from the DOM's computed style.
///
/// Falls back to `#000000` for any color that cannot be read or converted.
pub fn extract_theme_colors() -> ThemeColors {
    let win = window().expect("no global window");
    let doc = win.document().expect("no document on window");
    let root = doc.document_element().expect("no document element");

    let style = win
        .get_computed_style(&root)
        .ok()
        .flatten()
        .expect("getComputedStyle failed");

    let read = |name: &str| -> String {
        let raw = style
            .get_property_value(&format!("--color-{name}"))
            .unwrap_or_default();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return "#000000".to_string();
        }
        oklch_to_hex(trimmed).unwrap_or_else(|_| "#000000".to_string())
    };

    let theme_name = root
        .get_attribute("data-theme")
        .unwrap_or_else(|| "unknown".to_string());

    ThemeColors {
        theme_name,
        primary: read(COLOR_NAMES[0]),
        primary_content: read(COLOR_NAMES[1]),
        secondary: read(COLOR_NAMES[2]),
        secondary_content: read(COLOR_NAMES[3]),
        accent: read(COLOR_NAMES[4]),
        accent_content: read(COLOR_NAMES[5]),
        neutral: read(COLOR_NAMES[6]),
        neutral_content: read(COLOR_NAMES[7]),
        base_100: read(COLOR_NAMES[8]),
        base_200: read(COLOR_NAMES[9]),
        base_300: read(COLOR_NAMES[10]),
        base_content: read(COLOR_NAMES[11]),
        info: read(COLOR_NAMES[12]),
        info_content: read(COLOR_NAMES[13]),
        success: read(COLOR_NAMES[14]),
        success_content: read(COLOR_NAMES[15]),
        warning: read(COLOR_NAMES[16]),
        warning_content: read(COLOR_NAMES[17]),
        error: read(COLOR_NAMES[18]),
        error_content: read(COLOR_NAMES[19]),
    }
}
