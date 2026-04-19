use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-base-200 flex items-center justify-center">
            <div class="text-center">
                <h1 class="text-6xl font-extrabold text-base-content/30">"404"</h1>
                <p class="text-xl mt-4 text-base-content/70">"Page not found."</p>
                <a href="/" class="btn btn-primary mt-8">"Back to home"</a>
            </div>
        </div>
    }
}
