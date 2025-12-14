use crate::chat::ChatView;
use crate::map::MapView;
use crate::state::{GameContext, provide_game_context};
use leptos::either::Either;
use leptos::prelude::*;
use sint_core::{Action, GameAction, GamePhase, MetaAction, types::MapLayout};

#[component]
fn PhaseTracker(phase: GamePhase) -> impl IntoView {
    let phases = vec![
        (GamePhase::Lobby, "LOBBY"),
        (GamePhase::MorningReport, "MORNING"),
        (GamePhase::EnemyTelegraph, "TELEGRAPH"),
        (GamePhase::TacticalPlanning, "PLANNING"),
        (GamePhase::Execution, "EXECUTE"),
        (GamePhase::EnemyAction, "ENEMY"),
        (GamePhase::GameOver, "GAME OVER"),
        (GamePhase::Victory, "VICTORY!"),
    ];

    view! {
        <div style="display: flex; gap: 5px; align-items: center; font-size: 0.8em; background: #222; padding: 5px; border-radius: 4px;">
            {phases
                .into_iter()
                .map(|(p, label)| {
                    let active = p == phase;
                    let weight = if active { "bold" } else { "normal" };
                    view! {
                        <div style=format!(
                            "color: {}; font-weight: {}; padding: 2px 6px; border-radius: 2px; background: {};",
                            if active { "#fff" } else { "#888" },
                            weight,
                            if active { "#4caf50" } else { "transparent" },
                        )>{label}</div>
                        // Arrow separator (except last)
                        {if label != "ENEMY" { "→" } else { "" }}
                    }
                })
                .collect::<Vec<_>>()}
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
                    {move || {
                        if is_connected.get() {
                            view! {
                                <span style="color: #4caf50; font-size: 0.8em;">"● ONLINE"</span>
                            }
                        } else {
                            view! {
                                <span style="color: #f44336; font-size: 0.8em;">"⚠ OFFLINE"</span>
                            }
                        }
                    }}
                </div>

                {move || view! { <PhaseTracker phase=state.get().phase /> }}

                <div style="text-align: right; font-size: 0.9em;">
                    <span style="margin-right: 15px; font-weight: bold; color: #81c784;">
                        {move || pid.clone()}
                    </span>
                    <span title="Hull Integrity" style="margin-right: 10px;">
                        "🛡 "
                        {move || state.get().hull_integrity}
                    </span>
                    <span title="Turn Number">"⏳ T" {move || state.get().turn_count}</span>
                </div>
            </header>

            // --- MAIN CONTENT (Grid) ---
            <div style="flex: 1; display: grid; grid-template-columns: 300px 1fr 300px; gap: 1px; background: #333; overflow: hidden;">

                // LEFT PANEL: Status & Actions
                <div style="background: #2a2a2a; display: flex; flex-direction: column; overflow-y: auto; border-right: 1px solid #444;">
                    <div style="padding: 15px;">
                        <h3 style="margin-top: 0; border-bottom: 1px solid #444; padding-bottom: 5px;">
                            "Status Report"
                        </h3>
                        <MyStatus ctx=ctx.clone() />

                        <div style="margin-top: 15px;">
                            <MorningReportView ctx=ctx.clone() />
                        </div>
                    </div>

                    // Spacer
                    <div style="flex: 1;"></div>

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

                    // Game Over Overlay
                    {move || {
                        if state.get().phase == GamePhase::GameOver {
                            Either::Left(
                                view! {
                                    <div style="position: absolute; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 100;">
                                        <h1 style="color: #f44336; font-size: 3em; margin: 0; text-shadow: 0 0 10px #f44336;">
                                            "GAME OVER"
                                        </h1>
                                        <div style="color: white; font-size: 1.2em; margin-top: 10px;">
                                            "The Steamboat has fallen."
                                        </div>
                                        <div style="margin-top: 20px; color: #aaa;">
                                            "Refresh to try again."
                                        </div>
                                    </div>
                                },
                            )
                        } else if state.get().phase == GamePhase::Victory {
                            Either::Right(
                                Either::Left(
                                    view! {
                                        <div style="position: absolute; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 100;">
                                            <h1 style="color: #4caf50; font-size: 3em; margin: 0; text-shadow: 0 0 10px #4caf50;">
                                                "VICTORY!"
                                            </h1>
                                            <div style="color: white; font-size: 1.2em; margin-top: 10px;">
                                                "The Steamboat is safe... for now."
                                            </div>
                                            <div style="margin-top: 20px; color: #aaa;">
                                                "All Bosses Defeated."
                                            </div>
                                        </div>
                                    },
                                ),
                            )
                        } else {
                            Either::Right(Either::Right(()))
                        }
                    }}
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
                    Either::Left(
                        view! {
                            <div style="color: #666; font-style: italic;">
                                "No actions planned yet."
                            </div>
                        },
                    )
                } else {
                    Either::Right(
                        s
                            .proposal_queue
                            .iter()
                            .map(|p| {
                                let is_mine = p.player_id == pid;
                                let action_desc = format!("{:?}", p.action);
                                let c_undo = ctx_undo.clone();
                                let action_id = p.id;
                                let p_id_disp = p.player_id.clone();

                                view! {
                                    <div style="margin-bottom: 8px; background: #333; padding: 6px; border-radius: 4px; border-left: 3px solid #2196f3;">
                                        <div style="display: flex; justify-content: space-between; align-items: center;">
                                            <span style="font-weight: bold; color: #90caf9;">
                                                {p_id_disp}
                                            </span>
                                            {if is_mine {
                                                Either::Left(
                                                    view! {
                                                        <button
                                                            style="background: #f44336; border: none; color: white; padding: 2px 6px; border-radius: 2px; font-size: 0.7em; cursor: pointer;"
                                                            on:click=move |_| {
                                                                c_undo
                                                                    .perform_action
                                                                    .call(Action::Game(GameAction::Undo { action_id }))
                                                            }
                                                        >
                                                            "UNDO"
                                                        </button>
                                                    },
                                                )
                                            } else {
                                                Either::Right(())
                                            }}
                                        </div>
                                        <div style="color: #ccc; margin-top: 2px;">
                                            {action_desc}
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
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
            let name = e.name.clone();

            view! {
                <div style="flex: 0 0 100px; border-bottom: 1px solid #444; margin-bottom: 20px; padding: 10px; background: #221a1a; display: flex; justify-content: space-between; align-items: center;">
                    // Enemy Info
                    <div>
                        <h2 style="margin: 0; color: #ff5252; font-size: 1.1em;">{name}</h2>
                        <div style="margin-top: 5px; width: 200px; height: 10px; background: #444; border-radius: 5px; overflow: hidden;">
                            <div style=format!(
                                "width: {}%; height: 100%; background: #ff5252;",
                                hp_percent,
                            )></div>
                        </div>
                        <div style="font-size: 0.8em; margin-top: 2px; color: #aaa;">
                            {e.hp} " / " {e.max_hp} " HP"
                        </div>
                    </div>

                    // Attack Telegraph
                    <div style="text-align: right;">
                        {if let Some(attack) = &e.next_attack {
                            let room_name = attack
                                .target_room
                                .and_then(|tid| s.map.rooms.get(&tid))
                                .map(|r| r.name.as_str())
                                .unwrap_or("Unknown");
                            let target_id_str = attack
                                .target_room
                                .map(|id| id.to_string())
                                .unwrap_or_else(|| "?".to_owned());
                            let attack_debug = format!("{:?}", attack.effect);
                            Either::Left(
                                view! {
                                    <div style="color: #ff9800; font-weight: bold;">
                                        "⚠ WARNING: ATTACK IMMINENT"
                                    </div>
                                    <div style="font-size: 0.9em; margin-top: 5px;">
                                        "Target: "
                                        <span style="color: #fff;">
                                            {room_name} " (" {target_id_str} ")"
                                        </span>
                                    </div>
                                    <div style="font-size: 0.8em; color: #ccc;">
                                        "Effect: " {attack_debug}
                                    </div>
                                },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    <div style="color: #81c784;">"No active threat detected."</div>
                                },
                            )
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
                Either::Left(

                    view! {
                        <div style="background: #673ab7; padding: 15px; border-radius: 8px; margin-bottom: 20px; border: 2px solid #9575cd;">
                            <h3 style="margin-top: 0;">"Morning Report"</h3>

                            // Latest Event (Just Drawn)
                            {if let Some(card) = &s.latest_event {
                                let title = card.title.clone();
                                let description = card.description.clone();
                                Either::Left(
                                    view! {
                                        <div style="margin-bottom: 20px; background: #fff; color: #222; padding: 15px; border-radius: 4px; border-left: 5px solid #ff4081;">
                                            <div style="font-size: 0.8em; text-transform: uppercase; color: #888; font-weight: bold;">
                                                "Just Drawn"
                                            </div>
                                            <h2 style="margin: 5px 0;">{title}</h2>
                                            <div>{description}</div>
                                        </div>
                                    },
                                )
                            } else {
                                Either::Right(())
                            }}

                            // Active Situations
                            {if has_situations {
                                Either::Left(
                                    view! {
                                        <div>
                                            <h4 style="margin: 10px 0;">"Active Situations"</h4>
                                            <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                                {s
                                                    .active_situations
                                                    .iter()
                                                    .map(|card| {
                                                        let title = card.title.clone();
                                                        let description = card.description.clone();
                                                        view! {
                                                            <div style="background: #eee; color: #222; padding: 10px; border-radius: 4px; width: 200px;">
                                                                <strong>{title}</strong>
                                                                <div style="font-size: 0.8em; margin-top: 5px;">
                                                                    {description}
                                                                </div>
                                                                {if let Some(sol) = &card.solution {
                                                                    let sol_debug = format!("{:?}", sol);
                                                                    Either::Left(
                                                                        view! {
                                                                            <div style="font-size: 0.7em; margin-top: 5px; color: #555;">
                                                                                "Solution: " {sol_debug}
                                                                            </div>
                                                                        },
                                                                    )
                                                                } else {
                                                                    Either::Right(())
                                                                }}
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    },
                                )
                            } else {
                                Either::Right(())
                            }}
                        </div>
                    },
                )
            } else {
                Either::Right(())
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

    let (name_input, set_name_input) = signal(pid.clone());

    view! {
        <div style="background: #333; padding: 15px; border-radius: 8px;">
            {move || {
                let s = state.get();
                if let Some(real_p) = s.players.get(&pid) {
                    let mut p = real_p.clone();
                    for prop in &s.proposal_queue {
                        if prop.player_id == pid && let GameAction::Move { to_room } = prop.action {
                            p.room_id = to_room;
                        }
                    }
                    let room_name = s
                        .map
                        .rooms
                        .get(&p.room_id)
                        .map(|r| r.name.as_str())
                        .unwrap_or("Unknown");
                    let is_ready = p.is_ready;
                    let c_ready = ctx_ready.clone();
                    let is_lobby = s.phase == GamePhase::Lobby;
                    Either::Left(

                        view! {
                            <div style="display: flex; gap: 10px; flex-wrap: wrap; align-items: center;">
                                // Name Input or Display
                                {if is_lobby {
                                    let c_up = ctx_update.clone();
                                    let c_map = ctx_update.clone();
                                    let current_layout = s.layout;
                                    Either::Left(
                                        view! {
                                            <div style="width: 100%; display: flex; flex-direction: column; gap: 5px; margin-bottom: 5px;">
                                                <div style="display: flex; gap: 5px;">
                                                    <input
                                                        type="text"
                                                        prop:value=name_input
                                                        on:input=move |ev| {
                                                            set_name_input.set(event_target_value(&ev))
                                                        }
                                                        style="flex: 1; padding: 6px; border-radius: 4px; border: 1px solid #555; background: #222; color: white; min-width: 0;"
                                                    />
                                                    <button
                                                        on:click=move |_| {
                                                            c_up.perform_action
                                                                .call(
                                                                    Action::Meta(MetaAction::SetName {
                                                                        name: name_input.get(),
                                                                    }),
                                                                )
                                                        }
                                                        style="padding: 6px 12px; background: #2196f3; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                                    >
                                                        "UPDATE"
                                                    </button>
                                                </div>

                                                // Map Layout Selector
                                                <div style="display: flex; align-items: center; gap: 5px; background: #222; padding: 5px; border-radius: 4px;">
                                                    <span style="font-size: 0.9em; color: #aaa;">"Map:"</span>
                                                    <select
                                                        on:change=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            let layout = match val.as_str() {
                                                                "Torus" => MapLayout::Torus,
                                                                _ => MapLayout::Star,
                                                            };
                                                            c_map
                                                                .perform_action
                                                                .call(Action::Meta(MetaAction::SetMapLayout { layout }));
                                                        }
                                                        prop:value=format!("{:?}", current_layout)
                                                        style="flex: 1; padding: 4px; border-radius: 2px; border: 1px solid #555; background: #333; color: white;"
                                                    >
                                                        <option value="Star">"Star Layout"</option>
                                                        <option value="Torus">"Torus Layout"</option>
                                                    </select>
                                                </div>
                                            </div>
                                        },
                                    )
                                } else {
                                    let layout_name = format!("{:?}", s.layout);
                                    Either::Right(
                                        view! {
                                            <div style="width: 100%; margin-bottom: 5px;">
                                                <div style="font-size: 1.1em;">
                                                    "👤 " <strong>{p.name.clone()}</strong>
                                                </div>
                                                <div style="font-size: 0.8em; color: #888;">
                                                    "Map: " {layout_name}
                                                </div>
                                            </div>
                                        },
                                    )
                                }}
                                <div style="width: 100%; display: flex; flex-direction: column; gap: 5px;">
                                    <div>
                                        "📍 Location: " <strong>{room_name}</strong> " ("
                                        {p.room_id} ")"
                                    </div>
                                    <div style="display: flex; justify-content: space-between;">
                                        <span>
                                            "❤ HP: " <strong>{p.hp}</strong> "/"
                                            {sint_core::logic::MAX_PLAYER_HP}
                                        </span>
                                        <span>
                                            "⚡ AP: " <strong>{p.ap}</strong> "/"
                                            {sint_core::logic::MAX_PLAYER_AP}
                                        </span>
                                    </div>
                                </div>
                                <div style="width: 100%; display: flex; gap: 5px; align-items: center; background: #222; padding: 5px; border-radius: 4px;">
                                    "🎒 "
                                    {if p.inventory.is_empty() {
                                        Either::Left(
                                            view! {
                                                <span style="color: #666; font-style: italic;">
                                                    "Empty"
                                                </span>
                                            },
                                        )
                                    } else {
                                        Either::Right(
                                            p
                                                .inventory
                                                .iter()
                                                .enumerate()
                                                .map(|(i, item)| {
                                                    let (emoji, name) = match item {
                                                        sint_core::ItemType::Peppernut => ("🍪", "Peppernut"),
                                                        sint_core::ItemType::Extinguisher => {
                                                            ("🧯", "Extinguisher")
                                                        }
                                                        sint_core::ItemType::Keychain => ("🔑", "Keychain"),
                                                        sint_core::ItemType::Wheelbarrow => ("🛒", "Wheelbarrow"),
                                                        sint_core::ItemType::Mitre => ("🧢", "Mitre"),
                                                    };
                                                    let c_drop = ctx_update.clone();
                                                    let is_planning = s.phase == GamePhase::TacticalPlanning;
                                                    let cursor = if is_planning {
                                                        "pointer"
                                                    } else {
                                                        "default"
                                                    };
                                                    let title = if is_planning {
                                                        format!("{} (Click to Drop)", name)
                                                    } else {
                                                        name.to_owned()
                                                    };

                                                    view! {
                                                        <span
                                                            title=title
                                                            style=format!(
                                                                "font-size: 1.2em; cursor: {}; margin-right: 5px;",
                                                                cursor,
                                                            )
                                                            on:click=move |_| {
                                                                if is_planning {
                                                                    c_drop
                                                                        .perform_action
                                                                        .call(Action::Game(GameAction::Drop { item_index: i }));
                                                                }
                                                            }
                                                        >
                                                            {emoji}
                                                        </span>
                                                    }
                                                })
                                                .collect::<Vec<_>>(),
                                        )
                                    }}
                                </div>
                                {if is_lobby {
                                    let c_ready_click = c_ready.clone();
                                    Either::Left(
                                        view! {
                                            <button
                                                on:click=move |_| {
                                                    c_ready_click
                                                        .perform_action
                                                        .call(
                                                            Action::Game(GameAction::VoteReady {
                                                                ready: !is_ready,
                                                            }),
                                                        );
                                                }
                                                style=move || {
                                                    if is_ready {
                                                        "width: 100%; background: #4caf50; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; margin-top: 5px;"
                                                    } else {
                                                        "width: 100%; background: #e91e63; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; font-weight: bold; box-shadow: 0 0 10px rgba(233, 30, 99, 0.4); margin-top: 5px;"
                                                    }
                                                }
                                            >
                                                {if is_ready {
                                                    "✅ WAITING FOR OTHERS..."
                                                } else {
                                                    "🚀 VOTE START"
                                                }}
                                            </button>
                                        },
                                    )
                                } else {
                                    let c_ready_click = c_ready.clone();
                                    Either::Right(
                                        view! {
                                            <button
                                                on:click=move |_| {
                                                    c_ready_click
                                                        .perform_action
                                                        .call(
                                                            Action::Game(GameAction::VoteReady {
                                                                ready: !is_ready,
                                                            }),
                                                        );
                                                }
                                                style=move || {
                                                    if is_ready {
                                                        "width: 100%; background: #4caf50; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; margin-top: 5px;"
                                                    } else {
                                                        "width: 100%; background: #555; color: white; border: none; padding: 10px; border-radius: 4px; cursor: pointer; margin-top: 5px;"
                                                    }
                                                }
                                            >
                                                {if is_ready { "✅ READY" } else { "❌ NOT READY" }}
                                            </button>
                                        },
                                    )
                                }}
                            </div>
                        },
                    )
                } else {
                    Either::Right(
                        view! {
                            <div style="display: flex; flex-direction: column; gap: 10px; align-items: center; justify-content: center; height: 100px;">
                                <div style="color: #4caf50; font-weight: bold; font-size: 1.1em;">
                                    "⏳ Connecting to Ship..."
                                </div>
                                <div style="font-size: 0.8em; color: #888;">
                                    "Syncing Neural Link for " {pid.clone()}
                                </div>
                            </div>
                        },
                    )
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
                    let ctx_iter = ctx_action.clone();
                    let pid_iter = pid.clone();
                    if s.phase != GamePhase::TacticalPlanning {
                        return vec![
                            Either::Left(
                                view! {
                                    <div style="color: #888; font-style: italic;">
                                        "Waiting for Tactical Planning..."
                                    </div>
                                },
                            ),
                        ];
                    }
                    let valid_actions = sint_core::logic::actions::get_valid_actions(&s, &pid);
                    if valid_actions.is_empty() {
                        return vec![
                            Either::Left(
                                view! {
                                    <div style="color: #888; font-style: italic;">
                                        "No valid actions available."
                                    </div>
                                },
                            ),
                        ];
                    }
                    valid_actions
                        .into_iter()
                        .filter_map(move |a| {
                            if let Action::Game(ga) = a {
                                match ga {
                                    GameAction::Chat { .. }
                                    | GameAction::Undo { .. }
                                    | GameAction::VoteReady { .. } => None,
                                    _ => {
                                        Some(
                                            Either::Right(
                                                render_action_button(ctx_iter.clone(), &s, &pid_iter, ga),
                                            ),
                                        )
                                    }
                                }
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<Either<_, _>>>()
                }}
            </div>
        </div>
    }
}

fn render_action_button(
    ctx: GameContext,
    state: &sint_core::GameState,
    pid: &str,
    action: GameAction,
) -> impl IntoView + use<> {
    // Calculate cost using Core logic
    let cost = sint_core::logic::actions::action_cost(state, pid, &action);

    // Determine Label & Style
    let (label, color, border) = match &action {
        GameAction::Move { to_room } => {
            let room_name = state
                .map
                .rooms
                .get(to_room)
                .map(|r| r.name.as_str())
                .unwrap_or("?");
            (
                format!("Move to {} ({})", room_name, to_room),
                "#2196f3",
                "none",
            )
        }
        GameAction::Bake => ("Bake Peppernuts".to_owned(), "#ff9800", "none"),
        GameAction::Shoot => ("Fire Cannons!".to_owned(), "#f44336", "none"),
        GameAction::RaiseShields => ("Raise Shields".to_owned(), "#3f51b5", "none"),
        GameAction::EvasiveManeuvers => ("Evasive Maneuvers".to_owned(), "#00bcd4", "none"),
        GameAction::Lookout => ("Lookout".to_owned(), "#673ab7", "none"),
        GameAction::Extinguish => ("Extinguish Fire".to_owned(), "#607d8b", "none"),
        GameAction::Repair => ("Repair Leak".to_owned(), "#2196f3", "none"),
        GameAction::PickUp { item_type } => (format!("Pick Up {:?}", item_type), "#8bc34a", "none"),
        GameAction::Interact => {
            // Dynamic Label for Interact
            let mut lbl = "Interact".to_owned();
            if let Some(p) = state.players.get(pid)
                && let Some(room) = state.map.rooms.get(&p.room_id)
            {
                for card in &state.active_situations {
                    if let Some(sol) = &card.solution {
                        let matches = match sol.target_system {
                            Some(sys) => room.system == Some(sys),
                            None => true,
                        };
                        if matches {
                            lbl = format!("SOLVE: {}", card.title);
                            break;
                        }
                    }
                }
            }
            (lbl, "#9c27b0", "2px solid #e1bee7")
        }
        GameAction::Revive { target_player } => {
            let name = state
                .players
                .get(target_player)
                .map(|p| p.name.as_str())
                .unwrap_or(target_player);
            (format!("Revive {}", name), "#009688", "none")
        }
        GameAction::FirstAid { target_player } => {
            let name = state
                .players
                .get(target_player)
                .map(|p| p.name.as_str())
                .unwrap_or(target_player);
            (format!("Heal {}", name), "#e91e63", "none")
        }
        GameAction::Pass => ("End Turn (Pass)".to_owned(), "#555", "none"),
        _ => (format!("{:?}", action), "#777", "none"),
    };

    view! {
        <button
            style=format!(
                "padding: 10px; background: {}; border: {}; color: white; border-radius: 4px; cursor: pointer; margin-right: 5px; margin-bottom: 5px; opacity: 1.0;",
                color,
                border,
            )
            on:click=move |_| { ctx.perform_action.call(Action::Game(action.clone())) }
        >
            {label}
            " ("
            {cost}
            " AP)"
        </button>
    }
}
