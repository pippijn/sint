use wasm_bindgen_test::*;
use leptos::prelude::*;
use leptos::mount::mount_to_body;
use leptos_router::components::Router;
use sint_client::App;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_app_starts_with_router() {
    // This should not panic
    mount_to_body(|| view! {
        <Router>
            <App />
        </Router>
    });
}

#[wasm_bindgen_test]
#[should_panic(expected = "Tried to access reactive URL outside a <Router> component")]
fn test_app_panics_without_router() {
    // This should panic because use_query_map requires a Router context
    mount_to_body(|| view! { <App /> });
}
