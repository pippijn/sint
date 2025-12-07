use leptos::*;
use crate::state::{provide_game_context, GameContext};
use crate::map::MapView;
use crate::chat::ChatView;
use sint_core::{Action, SystemType, HazardType, GamePhase};

#[component]
fn PhaseTracker(phase: GamePhase) -> impl IntoView {
    let phases = vec![
        (GamePhase::MorningReport, "MORNING"),
        (GamePhase::EnemyTelegraph, "TELEGRAPH"),
        (GamePhase::TacticalPlanning, "PLANNING"),
        (GamePhase::Execution, "EXECUTE"),
        (GamePhase::EnemyAction, "ENEMY"),
    ];
    
    view! {
        <div style="display: flex; gap: 5px; align-items: center; font-size: 0.8em; background: #222; padding: 5px; border-radius: 4px;">
            {phases.into_iter().map(|(p, label)| {
                let active = p == phase;
                let color = if active { "#4caf50" } else { "#555" };
                let weight = if active { "bold" } else { "normal" };
                view! {
                    <div style=format!("color: {}; font-weight: {}; padding: 2px 6px; border-radius: 2px; background: {};", 
                        if active { "#fff" } else { "#888" }, 
                        weight,
                        if active { "#4caf50" } else { "transparent" }
                    )>
                        {label}
                    </div>
                    // Arrow separator (except last)
                    {if label != "ENEMY" { "→" } else { "" }}
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let ctx = provide_game_context();
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    let is_connected = ctx.is_connected;
    
    view! {
        <div style="padding: 20px; font-family: monospace; max-width: 1200px; margin: 0 auto;">
            <header style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; background: #1a1a1a; padding: 15px; border-radius: 8px; border: 1px solid #333;">
                <div style="display: flex; align-items: center; gap: 15px;">
                    <h1 style="margin: 0; font-size: 1.5em;">"Sint FTL"</h1>
                    {move || if is_connected.get() {
                        view! { <span style="color: #4caf50; font-size: 0.8em;">"● ONLINE"</span> }
                    } else {
                        view! { <span style="color: #f44336; font-size: 0.8em;">"⚠ OFFLINE"</span> }
                    }}
                </div>
                
                <div style="display: flex; flex-direction: column; align-items: center; gap: 5px;">
                    {move || view! { <PhaseTracker phase=state.get().phase /> }}
                </div>

                <div style="text-align: right; font-size: 0.9em;">
                    <div style="margin-bottom: 4px;"><strong>{pid}</strong></div>
                    <div style="display: flex; gap: 10px;">
                        <span title="Hull Integrity">"🛡 " {move || state.get().hull_integrity} "/20"</span>
                        <span title="Turn Number">"⏳ T" {move || state.get().turn_count}</span>
                    </div>
                </div>
            </header>
            
            <MyStatus ctx=ctx.clone() />
            
            <hr style="border-color: #444; margin: 20px 0;" />
            
            <MorningReportView ctx=ctx.clone() />
            
            <Actions ctx=ctx.clone() />

            <hr style="border-color: #444; margin: 20px 0;" />
            
            <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 20px;">
                <div>
                    <h3>"Ship Map"</h3>
                    <MapView ctx=ctx.clone() />
                </div>
                <div>
                    <h3>"Comms"</h3>
                    <ChatView ctx=ctx.clone() />
                </div>
            </div>
            
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
fn MorningReportView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    view! {
        {move || {
            let s = state.get();
            let has_event = s.latest_event.is_some();
            let has_situations = !s.active_situations.is_empty();
            
            if has_event || has_situations {
                view! {
                    <div style="background: #673ab7; padding: 15px; border-radius: 8px; margin-bottom: 20px; border: 2px solid #9575cd;">
                        <h3 style="margin-top: 0;">"Morning Report"</h3>
                        
                        // Latest Event (Just Drawn)
                        {if let Some(card) = &s.latest_event {
                            view! {
                                <div style="margin-bottom: 20px; background: #fff; color: #222; padding: 15px; border-radius: 4px; border-left: 5px solid #ff4081;">
                                    <div style="font-size: 0.8em; text-transform: uppercase; color: #888; font-weight: bold;">"Just Drawn"</div>
                                    <h2 style="margin: 5px 0;">{&card.title}</h2>
                                    <div>{&card.description}</div>
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}

                        // Active Situations
                        {if has_situations {
                            view! {
                                <div>
                                    <h4 style="margin: 10px 0;">"Active Situations"</h4>
                                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                        {s.active_situations.iter().map(|card| {
                                            view! {
                                                <div style="background: #eee; color: #222; padding: 10px; border-radius: 4px; width: 200px;">
                                                    <strong>{&card.title}</strong>
                                                    <div style="font-size: 0.8em; margin-top: 5px;">{&card.description}</div>
                                                    {if let Some(sol) = &card.solution {
                                                        view! { <div style="font-size: 0.7em; margin-top: 5px; color: #555;">"Solution: " {format!("{:?}", sol)}</div> }.into_view()
                                                    } else {
                                                        view! {}.into_view()
                                                    }}
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }
        }}
    }
}

#[component]
fn MyStatus(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    let ctx_join = ctx.clone();
    let pid_join = pid.clone();
    let ctx_ready = ctx.clone();
    let pid_ready = pid.clone();
    
    view! {
        <div style="background: #333; padding: 15px; border-radius: 8px;">
            {move || {
                let s = state.get();
                if let Some(p) = s.players.get(&pid) {
                    let room_name = s.map.rooms.get(&p.room_id).map(|r| r.name.clone()).unwrap_or("Unknown".to_string());
                    let is_ready = p.is_ready;
                    let c_ready = ctx_ready.clone();
                    
                    view! {
                        <div style="display: flex; gap: 20px; flex-wrap: wrap; align-items: center;">
                            <div>"📍 Location: " <strong>{room_name}</strong> " (" {p.room_id} ")"</div>
                            <div>"❤ HP: " <strong>{p.hp}</strong> "/3"</div>
                            <div>"⚡ AP: " <strong>{p.ap}</strong> "/2"</div>
                            <div>"🎒 Inventory: " {format!("{:?}", p.inventory)}</div>
                            <button
                                on:click=move |_| {
                                    c_ready.perform_action.call(Action::VoteReady { ready: !is_ready });
                                }
                                style=move || if is_ready {
                                    "background: #4caf50; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                } else {
                                    "background: #555; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                }
                            >
                                {if is_ready { "✅ READY" } else { "❌ NOT READY" }}
                            </button>
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
                         
                         // Contextual Interact (Solve Situation)
                         for card in &s.active_situations {
                             if let Some(sol) = &card.solution {
                                 let room_match = sol.room_id.map_or(true, |rid| rid == player.room_id);
                                 
                                 if room_match {
                                     let c_interact = ctx_action.clone();
                                     let title = card.title.clone();
                                     let cost = sol.ap_cost;
                                     
                                     buttons.push(view! {
                                         <button 
                                            style="padding: 10px; background: #9c27b0; border: none; color: white; border-radius: 4px; cursor: pointer; border: 2px solid #e1bee7;"
                                            on:click=move |_| c_interact.perform_action.call(Action::Interact)
                                        >
                                            "SOLVE: " {title} " (" {cost} " AP)"
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
                     }
                     
                     buttons
                }}
            </div>
        </div>
    }
}