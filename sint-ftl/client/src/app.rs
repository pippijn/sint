use crate::game::GameView;
use crate::lobby::LobbyBrowser;
use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    // Basic routing based on Query Params
    let location = web_sys::window().unwrap().location();
    let search = location.search().unwrap_or_default();
    let params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();

    let room_param = params.get("room");
    let player_param = params.get("player");

    match room_param {
        Some(rid) => {
            let pid = player_param
                .unwrap_or_else(|| format!("Player_{}", &uuid::Uuid::new_v4().to_string()[..5]));

            view! { <GameView room_id=rid player_id=pid /> }
        }
        None => {
            view! { <LobbyBrowser /> }
        }
    }
}
