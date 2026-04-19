/// The viewer HTML template. Placeholders are replaced by `compile_html`:
/// - `{{MANIFEST_JSON}}`  — serialized ViewerManifest as JSON
/// - `{{AUDIO_BASE64}}`   — Opus audio file encoded as Base64
/// - `{{SLIDE_SCRIPTS}}`  — `<script type="text/xml" id="slide-N">` blocks
pub const TEMPLATE: &str = include_str!("player_template.html");
