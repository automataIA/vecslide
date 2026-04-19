use crate::{DarkMode, LoadedFile};
use js_sys::Uint8Array;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use lucide_leptos::{Clock, FileText, Folder, GitBranch, Github, Moon, Plus, Sun, Upload};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{DragEvent, File, HtmlInputElement};

/// Reads a `web_sys::File` into bytes and stores them in the `LoadedFile` context,
/// then navigates to the editor.
async fn load_and_navigate(
    file: File,
    file_signal: RwSignal<Option<Vec<u8>>>,
    navigate: impl Fn(&str, NavigateOptions) + 'static,
) {
    match JsFuture::from(file.array_buffer()).await {
        Ok(ab) => {
            let bytes = Uint8Array::new(&ab).to_vec();
            file_signal.set(Some(bytes));
            navigate("/editor", Default::default());
        }
        Err(e) => leptos::logging::error!("Failed to read .vecslide file: {:?}", e),
    }
}

#[component]
pub fn Home() -> impl IntoView {
    let DarkMode(dark) = use_context::<DarkMode>().expect("DarkMode context missing");
    let LoadedFile(file_signal) = use_context::<LoadedFile>().expect("LoadedFile context missing");

    let drag_over = RwSignal::new(false);

    // ── File picker ──────────────────────────────────────────────────────────
    let navigate_open = use_navigate();
    let on_file_change = move |ev: leptos::ev::Event| {
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(input) = input
            && let Some(files) = input.files()
            && let Some(file) = files.get(0)
        {
            let nav = navigate_open.clone();
            leptos::task::spawn_local(async move {
                load_and_navigate(file, file_signal, nav).await;
            });
        }
    };

    // ── Drag & drop ──────────────────────────────────────────────────────────
    let on_dragover = move |ev: DragEvent| {
        ev.prevent_default();
        drag_over.set(true);
    };
    let on_dragleave = move |_: DragEvent| {
        drag_over.set(false);
    };
    let navigate_drop = use_navigate();
    let on_drop = move |ev: DragEvent| {
        ev.prevent_default();
        drag_over.set(false);
        if let Some(dt) = ev.data_transfer()
            && let Some(files) = dt.files()
            && let Some(file) = files.get(0)
        {
            let nav = navigate_drop.clone();
            leptos::task::spawn_local(async move {
                load_and_navigate(file, file_signal, nav).await;
            });
        }
    };

    // ── "Create new" ─────────────────────────────────────────────────────────
    let navigate_new = use_navigate();
    let on_new = move |_| {
        navigate_new("/editor", Default::default());
    };

    // ── "Try the editor" (Typst callout) ─────────────────────────────────────
    let navigate_try = use_navigate();
    let on_try = move |_| {
        navigate_try("/editor", Default::default());
    };

    view! {
        <div class="min-h-[100dvh] bg-base-200 bg-dot-grid flex flex-col">

            // ── Navbar ────────────────────────────────────────────────────
            <nav class="navbar bg-base-100 border-b border-base-300 px-6 sticky top-0 z-50">
                <div class="flex flex-1 items-end gap-2">
                    <img
                        src="/vecslide-logo-horizontal-color.svg"
                        alt="VecSlide"
                        class="h-9 w-auto select-none"
                        draggable="false"
                    />
                    <span class="badge badge-primary badge-sm mb-1">"alpha"</span>
                </div>
                <div class="flex-none gap-2">
                    <a
                        href="https://github.com/vecslide/vecslide"
                        class="btn btn-ghost btn-sm gap-2"
                        target="_blank"
                        rel="noopener noreferrer"
                        aria-label="Open on GitHub"
                    >
                        <Github size=16 />
                        "GitHub"
                    </a>

                    // ── Theme toggle: bumblebee (light) ↔ business (dark) ──
                    <label class="swap swap-rotate" aria-label="Toggle dark/light theme">
                        <input
                            type="checkbox"
                            prop:checked=move || dark.get()
                            on:change=move |ev| {
                                let checked = ev
                                    .target()
                                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                                    .map(|i| i.checked())
                                    .unwrap_or(false);
                                dark.set(checked);
                            }
                        />
                        // Sun — shown when light theme (bumblebee) is active
                        <div class="swap-off h-10 w-10 fill-current">
                            <Sun size=40 fill="currentColor" stroke_width=2 />
                        </div>
                        // Moon — shown when dark theme (business) is active
                        <div class="swap-on h-10 w-10 fill-current">
                            <Moon size=40 fill="currentColor" stroke_width=2 />
                        </div>
                    </label>
                </div>
            </nav>

            // ── Hero ──────────────────────────────────────────────────────
            <section class="hero flex-1 py-10 px-6">
                <div class="hero-content text-center max-w-3xl flex-col gap-8">
                    <div class="space-y-4">
                        <h1 class="text-6xl font-extrabold leading-tight tracking-tight">
                            "Present in "
                            <span class="text-primary">"vector."</span>
                        </h1>
                        <p class="text-lg text-base-content/80 max-w-lg mx-auto leading-relaxed">
                            "SVG slides synchronized with Opus audio. "
                            "No raster video, no rasterization. "
                            "A 13 MB HTML file equals one hour of 4K lecture."
                        </p>
                    </div>

                    // ── CTA buttons ───────────────────────────────────────
                    <div class="flex flex-wrap gap-4 justify-center">
                        <label
                            class="btn btn-primary btn-lg gap-2 cursor-pointer"
                            aria-label="Open .vecslide file"
                        >
                            <Folder size=20 />
                            "Open presentation"
                            <input
                                type="file"
                                class="hidden"
                                accept=".vecslide,.html"
                                on:change=on_file_change
                                aria-hidden="true"
                                tabindex="-1"
                            />
                        </label>

                        <button
                            class="btn btn-outline btn-lg gap-2"
                            on:click=on_new
                            aria-label="Create a new presentation"
                        >
                            <Plus size=20 />
                            "Create new"
                        </button>
                    </div>

                    // ── Drag & drop zone ──────────────────────────────────
                    <div
                        class=move || {
                            let base = "drop-zone rounded-2xl w-full max-w-lg py-10 px-6 text-center cursor-pointer select-none";
                            if drag_over.get() { format!("{base} drop-zone-active") } else { base.to_string() }
                        }
                        on:dragover=on_dragover
                        on:dragleave=on_dragleave
                        on:drop=on_drop
                        role="region"
                        aria-label="Drag-and-drop zone for .vecslide files"
                    >
                        <div class="w-10 mx-auto mb-3 text-base-content/70">
                            <Upload size=40 stroke_width=2 />
                        </div>
                        <p class="text-base-content/70 text-sm">
                            "Drop a "
                            <code class="text-primary font-mono">".vecslide"</code>
                            " file here"
                        </p>
                    </div>
                </div>
            </section>

            // ── Stats row ─────────────────────────────────────────────────
            <section class="py-4 bg-base-100/60">
                <div class="container mx-auto px-6">
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-6 text-center">
                        <div>
                            <div class="text-3xl font-extrabold text-primary">"70×"</div>
                            <div class="text-sm text-base-content/60 mt-1">"lighter than 4K MP4"</div>
                        </div>
                        <div>
                            <div class="text-3xl font-extrabold text-primary">"13 MB"</div>
                            <div class="text-sm text-base-content/60 mt-1">"per hour in 4K"</div>
                        </div>
                        <div>
                            <div class="text-3xl font-extrabold text-primary">"<1 ms"</div>
                            <div class="text-sm text-base-content/60 mt-1">"audio-slide sync"</div>
                        </div>
                        <div>
                            <div class="text-3xl font-extrabold text-primary">"SVG"</div>
                            <div class="text-sm text-base-content/60 mt-1">"infinitely scalable"</div>
                        </div>
                    </div>
                </div>
            </section>

            // ── Feature cards — asymmetric bento grid ──────────────────────
            <section class="container mx-auto px-6 py-8">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    // Featured card — spans full width on desktop
                    <div class="card-surface p-6 space-y-3 md:col-span-2 border-l-4 border-l-primary">
                        <div class="flex items-start gap-4">
                            <div class="w-10 h-10 rounded-xl bg-primary/15 flex items-center justify-center text-primary shrink-0" aria-hidden="true">
                                <Folder size=20 />
                            </div>
                            <div>
                                <h2 class="text-lg font-bold">"70× lighter"</h2>
                                <p class="text-base-content/80 text-sm leading-relaxed">
                                    "13 MB for one hour in 4K vs 930 MB for an MP4. "
                                    "SVG + Opus VBR: just text and audio."
                                </p>
                            </div>
                        </div>
                    </div>

                    <div class="card-surface p-6 space-y-3 border-l-4 border-l-accent">
                        <div class="w-10 h-10 rounded-xl bg-accent/15 flex items-center justify-center text-accent shrink-0" aria-hidden="true">
                            <Clock size=20 />
                        </div>
                        <h2 class="text-lg font-bold">"Millisecond sync"</h2>
                        <p class="text-base-content/80 text-sm leading-relaxed">
                            "Audio is the master clock. SVG animations, transitions "
                            "and pointer trail synchronized via Web Animations API."
                        </p>
                    </div>

                    <div class="card-surface p-6 space-y-3 border-l-4 border-l-secondary">
                        <div class="w-10 h-10 rounded-xl bg-primary/15 flex items-center justify-center text-primary shrink-0" aria-hidden="true">
                            <GitBranch size=20 />
                        </div>
                        <h2 class="text-lg font-bold">"Git-friendly"</h2>
                        <p class="text-base-content/80 text-sm leading-relaxed">
                            "Slides in SVG, timeline in YAML. "
                            "Human-readable diffs, merge without binary conflicts."
                        </p>
                    </div>
                </div>
            </section>

            // ── Typst callout ─────────────────────────────────────────────
            <section class="container mx-auto px-6 pb-8">
                <div class="card-surface p-8 flex flex-col md:flex-row items-center gap-6 text-center md:text-left">
                    <div class="w-14 h-14 rounded-2xl bg-primary/15 flex items-center justify-center text-primary shrink-0" aria-hidden="true">
                        <FileText size=28 />
                    </div>
                    <div class="flex-1 space-y-1">
                        <h3 class="font-bold text-lg">"Native Typst source"</h3>
                        <p class="text-base-content/70 text-sm">
                            "Write slides in Typst using the separator "
                            <code class="font-mono text-primary">"----"</code>
                            ". Math formulas, charts, vector text — live preview in the browser."
                        </p>
                    </div>
                    <button
                        class="btn btn-primary shrink-0"
                        on:click=on_try
                        aria-label="Open the Typst editor"
                    >
                        "Try the editor"
                    </button>
                </div>
            </section>

            // ── Footer ────────────────────────────────────────────────────
            <footer class="footer footer-center p-6 bg-base-100/60 text-base-content/50 text-xs border-t border-base-300/40">
                <p>"VecSlide — open-source vector presentation format"</p>
            </footer>

        </div>
    }
}
