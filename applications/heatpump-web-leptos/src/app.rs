use leptos::*;
use leptos_router::*;

use crate::components::layout::Layout;
use crate::components::Dashboard;
use crate::components::Settings;
use crate::state::provide_theme_context;

/// Main application component with routing
#[component]
pub fn App() -> impl IntoView {
    // Provide theme context at the app root
    provide_theme_context();

    view! {
        <Router>
            <Routes>
                <Route path="/" view=Layout>
                    <Route path="" view=|| view! { <Redirect path="/dashboard" /> } />
                    <Route path="dashboard" view=Dashboard />
                    <Route path="settings" view=Settings />
                </Route>
            </Routes>
        </Router>
    }
}
