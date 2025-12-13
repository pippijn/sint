use crate::game::GameView;
use crate::lobby::LobbyBrowser;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn App() -> impl IntoView {
    let query_map = use_query_map();
    let room_param = query_map.get().get("room");
    let player_param = query_map.get().get("player");

    match room_param {
        Some(rid) => {
            let pid = player_param
                .unwrap_or_else(|| format!("Player_{}", &uuid::Uuid::new_v4().to_string()[..5]));

            Either::Left(view! { <GameView room_id=rid player_id=pid /> })
        }
        None => Either::Right(view! { <LobbyBrowser /> }),
    }
}
