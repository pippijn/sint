use leptos::*;
use crate::state::{provide_game_context, GameContext};
use crate::map::MapView;
use crate::chat::ChatView;
use sint_core::{Action, SystemType, HazardType, GamePhase};

#[component]
fn PhaseTracker(phase: GamePhase) -> impl IntoView {
    let phases = vec![
        (GamePhase::Lobby, "LOBBY"),
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
pub fn GameView(room_id: String, player_id: String) -> impl IntoView {
    let ctx = provide_game_context(room_id, player_id);
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    let is_connected = ctx.is_connected;
    
    view! {
        <div style="display: flex; flex-direction: column; height: 100vh; background: #222; color: #eee; font-family: monospace; overflow: hidden;">
            // --- HEADER ---
            <header style="flex: 0 0 auto; display: flex; justify-content: space-between; align-items: center; padding: 10px 20px; background: #1a1a1a; border-bottom: 1px solid #333;">
                <div style="display: flex; align-items: center; gap: 15px;">
                    <h1 style="margin: 0; font-size: 1.2em; color: #fff;">"Sint FTL"</h1>
                    {move || if is_connected.get() {
                        view! { <span style="color: #4caf50; font-size: 0.8em;">"● ONLINE"</span> }
                    } else {
                        view! { <span style="color: #f44336; font-size: 0.8em;">"⚠ OFFLINE"</span> }
                    }}
                </div>
                
                {move || view! { <PhaseTracker phase=state.get().phase /> }}

                <div style="text-align: right; font-size: 0.9em;">
                    <span style="margin-right: 15px; font-weight: bold; color: #81c784;">{&pid}</span>
                    <span title="Hull Integrity" style="margin-right: 10px;">"🛡 " {move || state.get().hull_integrity}</span>
                    <span title="Turn Number">"⏳ T" {move || state.get().turn_count}</span>
                </div>
            </header>
            
            // --- MAIN CONTENT (Grid) ---
            <div style="flex: 1; display: grid; grid-template-columns: 300px 1fr 300px; gap: 1px; background: #333; overflow: hidden;">
                
                // LEFT PANEL: Status & Actions
                <div style="background: #2a2a2a; display: flex; flex-direction: column; overflow-y: auto; border-right: 1px solid #444;">
                    <div style="padding: 15px;">
                        <h3 style="margin-top: 0; border-bottom: 1px solid #444; padding-bottom: 5px;">"Status Report"</h3>
                        <MyStatus ctx=ctx.clone() />
                        
                        <div style="margin-top: 15px;">
                            <MorningReportView ctx=ctx.clone() />
                        </div>
                    </div>
                    
                    <div style="flex: 1;"></div> // Spacer
                    
                    <div style="padding: 15px; background: #222; border-top: 1px solid #444;">
                        <Actions ctx=ctx.clone() />
                    </div>
                </div>
                
                // CENTER PANEL: Map & Space
                <div style="background: #111; display: flex; flex-direction: column; padding: 20px; overflow: auto; position: relative;">
                    // Enemy View
                    <EnemyView ctx=ctx.clone() />
                    
                    // Map (Center)
                    <div style="flex: 1; display: flex; align-items: center; justify-content: center;">
                        <MapView ctx=ctx.clone() />
                    </div>
                </div>
                
                // RIGHT PANEL: Comms
                <div style="background: #2a2a2a; border-left: 1px solid #444; display: flex; flex-direction: column;">
                    <div style="flex: 1; border-bottom: 1px solid #444; overflow: hidden; display: flex; flex-direction: column;">
                         <div style="padding: 10px; background: #1a1a1a; font-weight: bold; border-bottom: 1px solid #444;">
                            "Tactical Plan"
                        </div>
                        <div style="flex: 1; overflow-y: auto;">
                            <ProposalQueueView ctx=ctx.clone() />
                        </div>
                    </div>
                    
                    <div style="height: 300px; display: flex; flex-direction: column;">
                        <div style="padding: 10px; background: #1a1a1a; font-weight: bold; border-bottom: 1px solid #444;">
                            "Comms Channel"
                        </div>
                        <div style="flex: 1; overflow: hidden;">
                            <ChatView ctx=ctx.clone() />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProposalQueueView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    let ctx_undo = ctx.clone();
    
    view! {
        <div style="padding: 10px; font-size: 0.9em;">
            {move || {
                let s = state.get();
                if s.proposal_queue.is_empty() {
                    view! { <div style="color: #666; font-style: italic;">"No actions planned yet."</div> }.into_view()
                } else {
                    s.proposal_queue.iter().map(|p| {
                         let is_mine = p.player_id == pid;
                         let action_desc = format!("{:?}", p.action);
                         let c_undo = ctx_undo.clone();
                         let action_id = p.id.clone();
                         
                         view! {
                             <div style="margin-bottom: 8px; background: #333; padding: 6px; border-radius: 4px; border-left: 3px solid #2196f3;">
                                 <div style="display: flex; justify-content: space-between; align-items: center;">
                                     <span style="font-weight: bold; color: #90caf9;">{&p.player_id}</span>
                                     {if is_mine {
                                         view! {
                                             <button 
                                                style="background: #f44336; border: none; color: white; padding: 2px 6px; border-radius: 2px; font-size: 0.7em; cursor: pointer;"
                                                on:click=move |_| c_undo.perform_action.call(Action::Undo { action_id: action_id.clone() })
                                             >
                                                "UNDO"
                                             </button>
                                         }.into_view()
                                     } else {
                                         view! {}.into_view()
                                     }}
                                 </div>
                                 <div style="color: #ccc; margin-top: 2px;">
                                     {action_desc}
                                 </div>
                             </div>
                         }
                    }).collect::<Vec<_>>().into_view()
                }
            }}
        </div>
    }
}

#[component]
fn EnemyView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    
    view! {
        {move || {
            let s = state.get();
            let e = &s.enemy;
            let hp_percent = (e.hp as f32 / e.max_hp as f32) * 100.0;
            let hp_percent = if hp_percent < 0.0 { 0.0 } else { hp_percent };
            
            view! {
                <div style="flex: 0 0 100px; border-bottom: 1px solid #444; margin-bottom: 20px; padding: 10px; background: #221a1a; display: flex; justify-content: space-between; align-items: center;">
                    // Enemy Info
                    <div>
                        <h2 style="margin: 0; color: #ff5252; font-size: 1.1em;">{&e.name}</h2>
                        <div style="margin-top: 5px; width: 200px; height: 10px; background: #444; border-radius: 5px; overflow: hidden;">
                            <div style=format!("width: {}%; height: 100%; background: #ff5252;", hp_percent)></div>
                        </div>
                        <div style="font-size: 0.8em; margin-top: 2px; color: #aaa;">
                            {e.hp} " / " {e.max_hp} " HP"
                        </div>
                    </div>
                    
                    // Attack Telegraph
                    <div style="text-align: right;">
                        {if let Some(attack) = &e.next_attack {
                             let room_name = s.map.rooms.get(&attack.target_room).map(|r| r.name.clone()).unwrap_or("Unknown".to_string());
                             view! {
                                 <div style="color: #ff9800; font-weight: bold;">"⚠ WARNING: ATTACK IMMINENT"</div>
                                 <div style="font-size: 0.9em; margin-top: 5px;">
                                     "Target: " <span style="color: #fff;">{room_name} " (" {attack.target_room} ")"</span>
                                 </div>
                                 <div style="font-size: 0.8em; color: #ccc;">
                                     "Effect: " {format!("{:?}", attack.effect)}
                                 </div>
                             }.into_view()
                        } else {
                            view! {
                                <div style="color: #81c784;">"No active threat detected."</div>
                            }.into_view()
                        }}
                    </div>
                </div>
            }
        }}
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
    let ctx_ready = ctx.clone();
    let ctx_update = ctx.clone();
    
    let (name_input, set_name_input) = create_signal(pid.clone());

    view! {
        <div style="background: #333; padding: 15px; border-radius: 8px;">
            {move || {
                let s = state.get();
                if let Some(real_p) = s.players.get(&pid) {
                    // PREDICTION LOGIC
                    let mut p = real_p.clone();
                    for prop in &s.proposal_queue {
                        if prop.player_id == pid {
                            if let Action::Move { to_room } = prop.action {
                                p.room_id = to_room;
                            }
                        }
                    }

                    let room_name = s.map.rooms.get(&p.room_id).map(|r| r.name.clone()).unwrap_or("Unknown".to_string());
                    let is_ready = p.is_ready;
                    let c_ready = ctx_ready.clone();
                    let is_lobby = s.phase == GamePhase::Lobby;
                    
                    view! {
                        <div style="display: flex; gap: 10px; flex-wrap: wrap; align-items: center;">
                            // Name Input or Display
                            {if is_lobby {
                                let c_up = ctx_update.clone();
                                view! {
                                    <div style="width: 100%; display: flex; gap: 5px; margin-bottom: 5px;">
                                        <input 
                                            type="text" 
                                            prop:value=name_input
                                            on:input=move |ev| set_name_input.set(event_target_value(&ev))
                                            style="flex: 1; padding: 6px; border-radius: 4px; border: 1px solid #555; background: #222; color: white; min-width: 0;"
                                        />
                                        <button
                                            on:click=move |_| c_up.perform_action.call(Action::SetName { name: name_input.get() })
                                            style="padding: 6px 12px; background: #2196f3; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                        >
                                            "UPDATE"
                                        </button>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div style="width: 100%; margin-bottom: 5px; font-size: 1.1em;">"👤 " <strong>{p.name}</strong></div> }.into_view()
                            }}

                            <div style="width: 100%; display: flex; flex-direction: column; gap: 5px;">
                                <div>"📍 Location: " <strong>{room_name}</strong> " (" {p.room_id} ")"</div>
                                <div style="display: flex; justify-content: space-between;">
                                    <span>"❤ HP: " <strong>{p.hp}</strong> "/3"</span>
                                    <span>"⚡ AP: " <strong>{p.ap}</strong> "/2"</span>
                                </div>
                            </div>
                            
                            <div style="width: 100%; display: flex; gap: 5px; align-items: center; background: #222; padding: 5px; border-radius: 4px;">
                                "🎒 " 
                                {if p.inventory.is_empty() {
                                    view! { <span style="color: #666; font-style: italic;">"Empty"</span> }.into_view()
                                } else {
                                    p.inventory.iter().map(|item| {
                                        let (emoji, name) = match item {
                                            sint_core::ItemType::Peppernut => ("🍪", "Peppernut"),
                                            sint_core::ItemType::Extinguisher => ("🧯", "Extinguisher"),
                                            sint_core::ItemType::Keychain => ("🔑", "Keychain"),
                                            sint_core::ItemType::Wheelbarrow => ("🛒", "Wheelbarrow"),
                                            sint_core::ItemType::Mitre => ("🧢", "Mitre"),
                                        };
                                        view! { <span title=name style="font-size: 1.2em; cursor: help;">{emoji}</span> }
                                    }).collect::<Vec<_>>().into_view()
                                }}
                            </div>
                            
                            {if is_lobby {
                                let c_ready_click = c_ready.clone();
                                view! {
                                     <button
                                        on:click=move |_| {
                                            c_ready_click.perform_action.call(Action::VoteReady { ready: !is_ready });
                                        }
                                        style=move || if is_ready {
                                            "width: 100%; background: #4caf50; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; margin-top: 5px;"
                                        } else {
                                            "width: 100%; background: #e91e63; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; font-weight: bold; box-shadow: 0 0 10px rgba(233, 30, 99, 0.4); margin-top: 5px;"
                                        }
                                    >
                                        {if is_ready { "✅ WAITING FOR OTHERS..." } else { "🚀 VOTE START" }}
                                    </button>
                                }.into_view()
                            } else {
                                let c_ready_click = c_ready.clone();
                                view! {
                                    <button
                                        on:click=move |_| {
                                            c_ready_click.perform_action.call(Action::VoteReady { ready: !is_ready });
                                        }
                                        style=move || if is_ready {
                                            "width: 100%; background: #4caf50; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; margin-top: 5px;"
                                        } else {
                                            "width: 100%; background: #555; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; margin-top: 5px;"
                                        }
                                    >
                                        {if is_ready { "✅ READY" } else { "❌ NOT READY" }}
                                    </button>
                                }.into_view()
                            }}
                        </div>
                    }.into_view()
                } else {
                    view! { 
                        <div style="display: flex; flex-direction: column; gap: 10px; align-items: center; justify-content: center; height: 100px;">
                            <div style="color: #4caf50; font-weight: bold; font-size: 1.1em;">"⏳ Connecting to Ship..."</div>
                            <div style="font-size: 0.8em; color: #888;">"Syncing Neural Link for " {pid.clone()}</div>
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
                     let mut buttons = vec![];
                     
                     if s.phase != GamePhase::TacticalPlanning {
                         return vec![view! { <div style="color: #888; font-style: italic;">"Waiting for Tactical Planning..."</div> }.into_view()];
                     }

                     if let Some(real_p) = s.players.get(&pid) {
                         // PREDICTION LOGIC
                         let mut player = real_p.clone();
                         for prop in &s.proposal_queue {
                             if prop.player_id == pid {
                                 if let Action::Move { to_room } = prop.action {
                                     player.room_id = to_room;
                                 }
                             }
                         }

                         if let Some(room) = s.map.rooms.get(&player.room_id) {
                            
                            // Move Buttons
                            for &neighbor in &room.neighbors {
                                let n_room = s.map.rooms.get(&neighbor);
                                let name = n_room.map(|r| r.name.clone()).unwrap_or("?".to_string());
                                let c_move = ctx_action.clone();
                                let disabled = player.ap < 1;
                                let opacity = if disabled { "0.5" } else { "1.0" };
                                let cursor = if disabled { "not-allowed" } else { "pointer" };
                                
                                buttons.push(view! {
                                    <button 
                                        style=format!("padding: 10px; background: #2196f3; border: none; color: white; border-radius: 4px; cursor: {}; opacity: {};", cursor, opacity)
                                        disabled=disabled
                                        on:click=move |_| c_move.perform_action.call(Action::Move { to_room: neighbor })
                                    >
                                        "Move to " {name} " (" {neighbor} ")"
                                    </button>
                                }.into_view());
                            }
                            
                            // Contextual Actions
                            if room.system == Some(SystemType::Kitchen) {
                                let c_bake = ctx_action.clone();
                                let disabled = player.ap < 1;
                                let opacity = if disabled { "0.5" } else { "1.0" };
                                let cursor = if disabled { "not-allowed" } else { "pointer" };
                                buttons.push(view! {
                                    <button 
                                        style=format!("padding: 10px; background: #ff9800; border: none; color: white; border-radius: 4px; cursor: {}; opacity: {};", cursor, opacity)
                                        disabled=disabled
                                        on:click=move |_| c_bake.perform_action.call(Action::Bake)
                                    >
                                        "Bake Peppernuts"
                                    </button>
                                }.into_view());
                            }
                            
                            if room.system == Some(SystemType::Cannons) {
                                let c_shoot = ctx_action.clone();
                                let disabled = player.ap < 1;
                                let opacity = if disabled { "0.5" } else { "1.0" };
                                let cursor = if disabled { "not-allowed" } else { "pointer" };
                                buttons.push(view! {
                                     <button 
                                        style=format!("padding: 10px; background: #f44336; border: none; color: white; border-radius: 4px; cursor: {}; opacity: {};", cursor, opacity)
                                        disabled=disabled
                                        on:click=move |_| c_shoot.perform_action.call(Action::Shoot)
                                    >
                                        "Fire Cannons!"
                                    </button>
                                }.into_view());
                            }
                            
                            if room.hazards.contains(&HazardType::Fire) {
                                let c_ext = ctx_action.clone();
                                let disabled = player.ap < 1;
                                let opacity = if disabled { "0.5" } else { "1.0" };
                                let cursor = if disabled { "not-allowed" } else { "pointer" };
                                buttons.push(view! {
                                     <button 
                                        style=format!("padding: 10px; background: #607d8b; border: none; color: white; border-radius: 4px; cursor: {}; opacity: {};", cursor, opacity)
                                        disabled=disabled
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
                                 let disabled = player.ap < 1;
                                 let opacity = if disabled { "0.5" } else { "1.0" };
                                 let cursor = if disabled { "not-allowed" } else { "pointer" };
                                 buttons.push(view! {
                                     <button 
                                        style=format!("padding: 10px; background: #8bc34a; border: none; color: black; border-radius: 4px; cursor: {}; opacity: {};", cursor, opacity)
                                        disabled=disabled
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
                                     let disabled = player.ap < cost as i32;
                                     let opacity = if disabled { "0.5" } else { "1.0" };
                                     let cursor = if disabled { "not-allowed" } else { "pointer" };
                                     
                                     buttons.push(view! {
                                         <button 
                                            style=format!("padding: 10px; background: #9c27b0; border: none; color: white; border-radius: 4px; cursor: {}; border: 2px solid #e1bee7; opacity: {};", cursor, opacity)
                                            disabled=disabled
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
