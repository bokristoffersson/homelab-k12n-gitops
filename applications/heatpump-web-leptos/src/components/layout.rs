use leptos::*;
use leptos_router::*;

use crate::state::{use_theme, Theme};

/// Layout component with navbar and content outlet
#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <div class="layout">
            <Navbar />
            <main class="main-content">
                <Outlet />
            </main>
        </div>
    }
}

/// Navbar with tabs and theme toggle
#[component]
fn Navbar() -> impl IntoView {
    let location = use_location();

    // Check if a path is active
    let is_active = move |path: &str| location.pathname.get().starts_with(path);

    view! {
        <nav class="navbar">
            <div class="navbar-content">
                <h1 class="navbar-title">"Heatpump Monitor"</h1>
                <div class="navbar-tabs">
                    <A
                        href="/dashboard"
                        class=move || if is_active("/dashboard") { "tab active" } else { "tab" }
                    >
                        "Dashboard"
                    </A>
                    <A
                        href="/settings"
                        class=move || if is_active("/settings") { "tab active" } else { "tab" }
                    >
                        "Settings"
                    </A>
                </div>
                <div class="navbar-actions">
                    <CurrentTime />
                    <ThemeToggle />
                </div>
            </div>
        </nav>
    }
}

/// Current time display that updates every second
#[component]
fn CurrentTime() -> impl IntoView {
    let (time, set_time) = create_signal(get_current_time());

    // Update time every second
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::callback::Interval;

        let interval = Interval::new(1000, move || {
            set_time.set(get_current_time());
        });

        on_cleanup(move || drop(interval));
    }

    view! {
        <span class="last-update">
            {move || time.get()}
        </span>
    }
}

/// Get the current time as a formatted string
fn get_current_time() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Date;
        let date = Date::new_0();
        format!(
            "{:02}:{:02}:{:02}",
            date.get_hours(),
            date.get_minutes(),
            date.get_seconds()
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::from("--:--:--")
    }
}

/// Theme toggle button
#[component]
fn ThemeToggle() -> impl IntoView {
    let theme_ctx = use_theme();

    let icon = move || {
        match theme_ctx.theme.get() {
            Theme::Light => "Dark", // Show what clicking will do
            Theme::Dark => "Light",
        }
    };

    view! {
        <button
            class="theme-toggle"
            aria-label="Toggle theme"
            on:click=move |_| theme_ctx.toggle()
        >
            {icon}
        </button>
    }
}
