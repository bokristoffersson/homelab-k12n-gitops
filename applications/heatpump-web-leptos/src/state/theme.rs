use leptos::*;

const STORAGE_KEY: &str = "theme";

/// Theme variants
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// Convert to string for storage and data attribute
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }

    /// Toggle between light and dark
    pub fn toggle(&self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}

/// Theme context containing the current theme and toggle function
#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: ReadSignal<Theme>,
    pub set_theme: WriteSignal<Theme>,
}

impl ThemeContext {
    /// Toggle between light and dark theme
    pub fn toggle(&self) {
        self.set_theme.update(|t| *t = t.toggle());
    }
}

/// Get the initial theme from localStorage or system preference
fn get_initial_theme() -> Theme {
    // Try localStorage first
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        if let Ok(Some(saved)) = storage.get_item(STORAGE_KEY) {
            if let Some(theme) = Theme::from_str(&saved) {
                return theme;
            }
        }
    }

    // Check system preference
    if let Some(window) = web_sys::window() {
        if let Ok(Some(media_query)) = window.match_media("(prefers-color-scheme: dark)") {
            if media_query.matches() {
                return Theme::Dark;
            }
        }
    }

    Theme::Light
}

/// Save theme to localStorage
fn save_theme(theme: Theme) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item(STORAGE_KEY, theme.as_str());
    }
}

/// Apply theme to document root element
fn apply_theme(theme: Theme) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(root) = document.document_element() {
            let _ = root.set_attribute("data-theme", theme.as_str());
        }
    }
}

/// Provide theme context to the application
/// Call this at the root of your app (e.g., in App component)
pub fn provide_theme_context() {
    let initial_theme = get_initial_theme();

    // Apply initial theme immediately
    apply_theme(initial_theme);

    let (theme, set_theme) = create_signal(initial_theme);

    // Effect to apply theme changes and save to localStorage
    create_effect(move |_| {
        let current_theme = theme.get();
        apply_theme(current_theme);
        save_theme(current_theme);
    });

    provide_context(ThemeContext { theme, set_theme });
}

/// Hook to access theme context
pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>().expect("ThemeContext must be provided by a parent component")
}
