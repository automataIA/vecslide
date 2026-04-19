//! DaisyUI theme color extraction and conversion.
//!
//! Provides [`ThemeColors`] — a palette of 19 semantic colors (hex `#rrggbb`)
//! extracted from DaisyUI CSS custom properties. Used by both the HTML viewer
//! (as CSS custom properties) and the Typst preamble (as `rgb("#...")` values).

use serde::{Deserialize, Serialize};

/// All 19 DaisyUI semantic colors stored as hex strings (`#rrggbb`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub theme_name: String,
    pub primary: String,
    pub primary_content: String,
    pub secondary: String,
    pub secondary_content: String,
    pub accent: String,
    pub accent_content: String,
    pub neutral: String,
    pub neutral_content: String,
    pub base_100: String,
    pub base_200: String,
    pub base_300: String,
    pub base_content: String,
    pub info: String,
    pub info_content: String,
    pub success: String,
    pub success_content: String,
    pub warning: String,
    pub warning_content: String,
    pub error: String,
    pub error_content: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl ThemeColors {
    /// Dark theme defaults matching DaisyUI "business" theme.
    pub fn dark_default() -> Self {
        Self {
            theme_name: "business".to_string(),
            primary: "#1c4f82".to_string(),
            primary_content: "#d6e4f0".to_string(),
            secondary: "#7c3aed".to_string(),
            secondary_content: "#e4d8fd".to_string(),
            accent: "#e68a00".to_string(),
            accent_content: "#140c00".to_string(),
            neutral: "#23282f".to_string(),
            neutral_content: "#a6adba".to_string(),
            base_100: "#1f2937".to_string(),
            base_200: "#111827".to_string(),
            base_300: "#0f1623".to_string(),
            base_content: "#d5d9de".to_string(),
            info: "#3abff8".to_string(),
            info_content: "#002b3d".to_string(),
            success: "#36d399".to_string(),
            success_content: "#003320".to_string(),
            warning: "#fbbd23".to_string(),
            warning_content: "#382800".to_string(),
            error: "#f87272".to_string(),
            error_content: "#470000".to_string(),
        }
    }

    /// Light theme defaults matching DaisyUI "bumblebee" theme.
    pub fn light_default() -> Self {
        Self {
            theme_name: "bumblebee".to_string(),
            primary: "#e0a82e".to_string(),
            primary_content: "#181400".to_string(),
            secondary: "#f9d72f".to_string(),
            secondary_content: "#181400".to_string(),
            accent: "#e0a82e".to_string(),
            accent_content: "#181400".to_string(),
            neutral: "#1f2937".to_string(),
            neutral_content: "#d5d9de".to_string(),
            base_100: "#ffffff".to_string(),
            base_200: "#f2f2f2".to_string(),
            base_300: "#e5e6e6".to_string(),
            base_content: "#1f2937".to_string(),
            info: "#3abff8".to_string(),
            info_content: "#002b3d".to_string(),
            success: "#36d399".to_string(),
            success_content: "#003320".to_string(),
            warning: "#fbbd23".to_string(),
            warning_content: "#382800".to_string(),
            error: "#f87272".to_string(),
            error_content: "#470000".to_string(),
        }
    }

    /// Generates CSS custom properties for the HTML viewer's `:root` block.
    pub fn to_viewer_css(&self) -> String {
        format!(
            "\
--vs-primary: {primary};\n  \
--vs-primary-content: {primary_content};\n  \
--vs-secondary: {secondary};\n  \
--vs-accent: {accent};\n  \
--vs-neutral: {neutral};\n  \
--vs-neutral-content: {neutral_content};\n  \
--vs-base-100: {base_100};\n  \
--vs-base-200: {base_200};\n  \
--vs-base-300: {base_300};\n  \
--vs-base-content: {base_content};\n  \
--vs-error: {error};",
            primary = self.primary,
            primary_content = self.primary_content,
            secondary = self.secondary,
            accent = self.accent,
            neutral = self.neutral,
            neutral_content = self.neutral_content,
            base_100 = self.base_100,
            base_200 = self.base_200,
            base_300 = self.base_300,
            base_content = self.base_content,
            error = self.error,
        )
    }
}

// ── OKLCH → hex conversion ─────────────────────────────────────────────────────

/// Parse a CSS `oklch(L C H)` string and convert to `#rrggbb` hex.
///
/// Accepts both `oklch(0.7 0.15 210)` and `oklch(70% 0.15 210)` forms.
/// Returns `Err` if the string cannot be parsed.
pub fn oklch_to_hex(s: &str) -> Result<String, String> {
    let s = s.trim();

    // Handle passthrough for already-hex values
    if s.starts_with('#') && (s.len() == 7 || s.len() == 4) {
        return Ok(s.to_string());
    }

    // Handle rgb() passthrough
    if s.starts_with("rgb(") {
        return rgb_str_to_hex(s);
    }

    let inner = s
        .strip_prefix("oklch(")
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("not an oklch() value: {s}"))?
        .trim();

    // Split on whitespace or commas
    let parts: Vec<&str> = inner.split(&[' ', ',', '/'] as &[char])
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();

    // Take first 3 components (ignore alpha if present)
    if parts.len() < 3 {
        return Err(format!("oklch needs 3 components, got {}: {s}", parts.len()));
    }

    let l = parse_lightness(parts[0])?;
    let c: f64 = parts[1].parse().map_err(|e| format!("bad chroma: {e}"))?;
    let h_deg: f64 = parts[2]
        .strip_suffix("deg")
        .unwrap_or(parts[2])
        .parse()
        .map_err(|e| format!("bad hue: {e}"))?;

    let (r, g, b) = oklch_to_srgb(l, c, h_deg);
    Ok(format!("#{:02x}{:02x}{:02x}", r, g, b))
}

fn parse_lightness(s: &str) -> Result<f64, String> {
    if let Some(pct) = s.strip_suffix('%') {
        let v: f64 = pct.parse().map_err(|e| format!("bad lightness: {e}"))?;
        Ok(v / 100.0)
    } else {
        let v: f64 = s.parse().map_err(|e| format!("bad lightness: {e}"))?;
        // Values > 1 are likely percentages without the % sign
        if v > 1.0 { Ok(v / 100.0) } else { Ok(v) }
    }
}

fn rgb_str_to_hex(s: &str) -> Result<String, String> {
    let inner = s
        .strip_prefix("rgb(")
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("not an rgb() value: {s}"))?
        .trim();
    let parts: Vec<&str> = inner.split(&[' ', ','] as &[char])
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() < 3 {
        return Err(format!("rgb needs 3 components: {s}"));
    }
    let r: u8 = parts[0].parse().map_err(|e| format!("bad r: {e}"))?;
    let g: u8 = parts[1].parse().map_err(|e| format!("bad g: {e}"))?;
    let b: u8 = parts[2].parse().map_err(|e| format!("bad b: {e}"))?;
    Ok(format!("#{r:02x}{g:02x}{b:02x}"))
}

/// Convert OKLCH (L in 0..1, C >= 0, H in degrees) to sRGB (0..255 each).
fn oklch_to_srgb(l: f64, c: f64, h_deg: f64) -> (u8, u8, u8) {
    let h_rad = h_deg.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    let (lr, lg, lb) = oklab_to_linear_srgb(l, a, b);
    let r = linear_to_srgb(lr);
    let g = linear_to_srgb(lg);
    let b = linear_to_srgb(lb);
    (r, g, b)
}

/// Convert Oklab (L, a, b) to linear sRGB.
fn oklab_to_linear_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    // Oklab → LMS (cube roots)
    let l_ = l + 0.396_337_792_3 * a + 0.215_803_758_1 * b;
    let m_ = l - 0.105_561_346_2 * a - 0.063_854_174_8 * b;
    let s_ = l - 0.089_484_178_1 * a - 1.291_485_548_0 * b;

    // Cube to get LMS
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // LMS → linear sRGB
    let r = 4.076_741_662_0 * l3 - 3.307_711_590_4 * m3 + 0.230_969_928_4 * s3;
    let g = -1.268_438_005_0 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_1 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3;

    (r, g, b)
}

/// Apply sRGB gamma and clamp to 0..255.
fn linear_to_srgb(c: f64) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_oklch() {
        assert_eq!(oklch_to_hex("oklch(0 0 0)").unwrap(), "#000000");
    }

    #[test]
    fn white_oklch() {
        assert_eq!(oklch_to_hex("oklch(1 0 0)").unwrap(), "#ffffff");
    }

    #[test]
    fn percentage_lightness() {
        assert_eq!(oklch_to_hex("oklch(0% 0 0)").unwrap(), "#000000");
        assert_eq!(oklch_to_hex("oklch(100% 0 0)").unwrap(), "#ffffff");
    }

    #[test]
    fn known_blue() {
        // oklch(0.5 0.2 260) should produce a blue-ish color
        let hex = oklch_to_hex("oklch(0.5 0.2 260)").unwrap();
        // Verify it's a valid hex and blue-dominant
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap();
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
        assert!(b > r, "expected blue > red for hue 260, got {hex}");
    }

    #[test]
    fn known_red() {
        // oklch(0.6 0.25 25) should produce a red-ish color
        let hex = oklch_to_hex("oklch(0.6 0.25 25)").unwrap();
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap();
        assert!(r > b, "expected red > blue for hue 25, got {hex}");
    }

    #[test]
    fn hex_passthrough() {
        assert_eq!(oklch_to_hex("#ff00aa").unwrap(), "#ff00aa");
    }

    #[test]
    fn rgb_passthrough() {
        assert_eq!(oklch_to_hex("rgb(255, 0, 128)").unwrap(), "#ff0080");
    }

    #[test]
    fn with_deg_suffix() {
        let hex = oklch_to_hex("oklch(0.7 0.15 210deg)").unwrap();
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
    }

    #[test]
    fn with_alpha_ignored() {
        // oklch with alpha channel — we ignore the 4th component
        let hex = oklch_to_hex("oklch(0.5 0.2 260 / 0.8)").unwrap();
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
    }

    #[test]
    fn viewer_css_contains_properties() {
        let theme = ThemeColors::dark_default();
        let css = theme.to_viewer_css();
        assert!(css.contains("--vs-primary:"));
        assert!(css.contains("--vs-base-100:"));
        assert!(css.contains("--vs-base-content:"));
        assert!(css.contains("--vs-error:"));
    }
}
