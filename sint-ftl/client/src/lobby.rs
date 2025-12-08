use leptos::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct RoomList {
    rooms: Vec<String>,
}

#[component]
pub fn LobbyBrowser() -> impl IntoView {
    // Identity State
    let (player_name, set_player_name) = create_signal(
         // Default random name
         format!("Player_{}", &uuid::Uuid::new_v4().to_string()[..5])
    );
    
    // New Room State
    let (new_room_name, set_new_room_name) = create_signal("Room_A".to_string());
    
    // Fetch Rooms
    let rooms_resource = create_resource(
        || (),
        move |_| async move {
            let location = web_sys::window().unwrap().location();
            let host = location.host().unwrap();
            
            let url = if host.contains(":8080") {
                "http://localhost:3000/api/rooms"
            } else {
                 "/api/rooms"
            };
            
            match Request::get(url).send().await {
                 Ok(resp) => {
                     if resp.ok() {
                         resp.json::<RoomList>().await.ok()
                     } else {
                         None
                     }
                 },
                 Err(_) => None
            }
        }
    );
    
    let join_game = move |room: String| {
        let p = player_name.get();
        if !p.is_empty() {
             let url = format!("?room={}&player={}", room, p);
             let _ = web_sys::window().unwrap().location().set_href(&url);
        }
    };

    view! {
        <div style="padding: 20px; font-family: monospace; color: white; background: #222; min-height: 100vh;">
            <h1>"Sint FTL - Lobby"</h1>
            
            <div style="margin-bottom: 20px; border: 1px solid #444; padding: 15px; border-radius: 8px;">
                <h3>"Identity"</h3>
                <input 
                    type="text" 
                    prop:value=player_name
                    on:input=move |ev| set_player_name.set(event_target_value(&ev))
                    style="padding: 8px; border-radius: 4px; border: 1px solid #555; background: #333; color: white;"
                />
            </div>

            <div style="display: flex; gap: 20px; flex-wrap: wrap;">
                // Create New
                <div style="flex: 1; min-width: 300px; border: 1px solid #444; padding: 15px; border-radius: 8px;">
                    <h3>"Create New Game"</h3>
                    <div style="display: flex; gap: 10px;">
                        <input 
                            type="text" 
                            prop:value=new_room_name
                            on:input=move |ev| set_new_room_name.set(event_target_value(&ev))
                            style="padding: 8px; border-radius: 4px; border: 1px solid #555; background: #333; color: white;"
                        />
                        <button
                            on:click=move |_| join_game(new_room_name.get())
                            style="padding: 8px 16px; background: #4caf50; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        >
                            "CREATE & JOIN"
                        </button>
                    </div>
                </div>

                // Join Existing
                <div style="flex: 1; min-width: 300px; border: 1px solid #444; padding: 15px; border-radius: 8px;">
                    <h3>"Existing Games"</h3>
                    <Suspense fallback=move || view! { "Loading..." }>
                        {move || {
                            rooms_resource.get().map(|data| {
                                match data {
                                    Some(list) => {
                                        if list.rooms.is_empty() {
                                            view! { <div style="color: #888;">"No active games found."</div> }.into_view()
                                        } else {
                                            list.rooms.into_iter().map(|r| {
                                                let r_clone = r.clone();
                                                view! {
                                                    <div style="margin-bottom: 5px; display: flex; justify-content: space-between; align-items: center; background: #333; padding: 8px; border-radius: 4px;">
                                                        <span>{r.clone()}</span>
                                                        <button
                                                            on:click=move |_| join_game(r_clone.clone())
                                                            style="padding: 4px 10px; background: #2196f3; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                                        >
                                                            "JOIN"
                                                        </button>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_view()
                                        }
                                    },
                                    None => view! { <div style="color: #f44336;">"Failed to load rooms."</div> }.into_view()
                                }
                            })
                        }}
                    </Suspense>
                    <button 
                        on:click=move |_| rooms_resource.refetch() 
                        style="margin-top: 10px; padding: 4px 8px; background: #555; color: white; border: none; border-radius: 2px; cursor: pointer;"
                    >
                        "↻ Refresh"
                    </button>
                </div>
            </div>
        </div>
    }
}
