use leptos::*;
use crate::state::{provide_game_context, GameContext};
use crate::map::MapView;
use sint_core::{Action, SystemType, HazardType};

#[component]
pub fn App() -> impl IntoView {
    let ctx = provide_game_context();
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    let is_connected = ctx.is_connected;
    
    view! {
        <div style="padding: 20px; font-family: monospace; max-width: 1200px; margin: 0 auto;">
            <header style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <div style="display: flex; align-items: center; gap: 15px;">
                    <h1>"Sint FTL"</h1>
                    {move || if is_connected.get() {
                        view! { <span style="color: #4caf50; font-weight: bold;">"● Connected"</span> }
                    } else {
                        view! { <span style="color: #f44336; font-weight: bold;">"⚠ Disconnected"</span> }
                    }}
                </div>
                <div style="text-align: right;">
                    <div><strong>"Player: "</strong> {&pid}</div>
                    <div><strong>"Phase: "</strong> {move || format!("{:?}", state.get().phase)}</div>
                    <div><strong>"Hull: "</strong> {move || state.get().hull_integrity} " / 20"</div>
                    <div><strong>"Turn: "</strong> {move || state.get().turn_count}</div>
                </div>
            </header>
            
            <MyStatus ctx=ctx.clone() />
            
            <hr style="border-color: #444; margin: 20px 0;" />
            
            <Actions ctx=ctx.clone() />

            <hr style="border-color: #444; margin: 20px 0;" />
            
            <h3>"Ship Map"</h3>
            <MapView ctx=ctx.clone() />
            
            <hr style="border-color: #444; margin: 20px 0;" />
            
            <details>
                <summary style="cursor: pointer; color: #888;">"Debug State (Click to Expand)"</summary>
                <pre style="background: #111; padding: 10px; overflow: auto; max-height: 400px; font-size: 0.8em; color: #aaa;">
                    {move || serde_json::to_string_pretty(&state.get()).unwrap_or_default()}
                </pre>
            </details>
        </div>
    }
}

#[component]
fn MyStatus(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    let ctx_join = ctx.clone();
    let pid_join = pid.clone();
    
    view! {
        <div style="background: #333; padding: 15px; border-radius: 8px;">
            {move || {
                let s = state.get();
                if let Some(p) = s.players.get(&pid) {
                    let room_name = s.map.rooms.get(&p.room_id).map(|r| r.name.clone()).unwrap_or("Unknown".to_string());
                    view! {
                        <div style="display: flex; gap: 20px; flex-wrap: wrap;">
                            <div>"📍 Location: " <strong>{room_name}</strong> " (" {p.room_id} ")"</div>
                            <div>"❤ HP: " <strong>{p.hp}</strong> "/3"</div>
                            <div>"⚡ AP: " <strong>{p.ap}</strong> "/2"</div>
                            <div>"🎒 Inventory: " {format!("{:?}", p.inventory)}</div>
                            <div>"Ready: " {if p.is_ready { "✅" } else { "❌" }}</div>
                        </div>
                    }.into_view()
                } else {
                    let c_join = ctx_join.clone();
                    let p_join = pid_join.clone();
                    view! { 
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <span style="color: #f44336;">"⚠ Not Joined"</span>
                            <button 
                                style="padding: 8px 16px; background: #4caf50; border: none; color: white; border-radius: 4px; cursor: pointer; font-weight: bold;"
                                on:click=move |_| c_join.perform_action.call(Action::Join { name: p_join.clone() })
                            >
                                "JOIN GAME"
                            </button>
                        </div> 
                    }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn Actions(ctx: GameContext) -> impl IntoView {
    let ctx_action = ctx.clone();
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    
    view! {
        <div>
            <h3>"Actions"</h3>
            <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                {move || {
                     let s = state.get();
                     let p = s.players.get(&pid);
                     let mut buttons = vec![];
                     
                     if let Some(player) = p {
                         if let Some(room) = s.map.rooms.get(&player.room_id) {
                            
                            // Move Buttons
                            for &neighbor in &room.neighbors {
                                let n_room = s.map.rooms.get(&neighbor);
                                let name = n_room.map(|r| r.name.clone()).unwrap_or("?".to_string());
                                let c_move = ctx_action.clone();
                                buttons.push(view! {
                                    <button 
                                        style="padding: 10px; background: #2196f3; border: none; color: white; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| c_move.perform_action.call(Action::Move { to_room: neighbor })
                                    >
                                        "Move to " {name} " (" {neighbor} ")"
                                    </button>
                                }.into_view());
                            }
                            
                            // Contextual Actions
                            if room.system == Some(SystemType::Kitchen) {
                                let c_bake = ctx_action.clone();
                                buttons.push(view! {
                                    <button 
                                        style="padding: 10px; background: #ff9800; border: none; color: white; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| c_bake.perform_action.call(Action::Bake)
                                    >
                                        "Bake Peppernuts"
                                    </button>
                                }.into_view());
                            }
                            
                            if room.system == Some(SystemType::Cannons) {
                                let c_shoot = ctx_action.clone();
                                buttons.push(view! {
                                     <button 
                                        style="padding: 10px; background: #f44336; border: none; color: white; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| c_shoot.perform_action.call(Action::Shoot)
                                    >
                                        "Fire Cannons!"
                                    </button>
                                }.into_view());
                            }
                            
                            if room.hazards.contains(&HazardType::Fire) {
                                let c_ext = ctx_action.clone();
                                buttons.push(view! {
                                     <button 
                                        style="padding: 10px; background: #607d8b; border: none; color: white; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| c_ext.perform_action.call(Action::Extinguish)
                                    >
                                        "Extinguish Fire"
                                    </button>
                                }.into_view());
                            }
                            
                            // Item Pickup
                            if !room.items.is_empty() {
                                 // Just pick up the first one for now (simplification)
                                 let c_pick = ctx_action.clone();
                                 let item = room.items[0].clone();
                                 buttons.push(view! {
                                     <button 
                                        style="padding: 10px; background: #8bc34a; border: none; color: black; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| c_pick.perform_action.call(Action::PickUp { item_index: 0 })
                                    >
                                        "Pick Up " {format!("{:?}", item)}
                                    </button>
                                }.into_view());
                            }
                         }
                     }
                     
                     // Pass
                     let c_pass = ctx_action.clone();
                     buttons.push(view! {
                         <button 
                            style="padding: 10px; background: #555; border: none; color: white; border-radius: 4px; cursor: pointer;"
                            on:click=move |_| c_pass.perform_action.call(Action::Pass)
                        >
                            "End Turn (Pass)"
                        </button>
                     }.into_view());
                     
                     buttons
                }}
            </div>
        </div>
    }
}