use crate::state::GameContext;
use leptos::either::Either;
use leptos::prelude::*;
use sint_core::{
    logic::pathfinding::find_path, types::MapLayout, Action, AttackEffect, GameAction, GamePhase,
    HazardType, ItemType, Player, Room,
};

#[derive(Clone, Copy, PartialEq)]
pub enum DoorDirection {
    Top,
    Bottom,
    Left,
    Right,
}

#[component]
pub fn MapView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;

    view! {
        {move || {
            let layout_type = state.get().layout;
            match layout_type {
                MapLayout::Star => Either::Left(view! { <StarMapView ctx=ctx.clone() /> }),
                MapLayout::Torus => Either::Right(view! { <TorusMapView ctx=ctx.clone() /> }),
            }
        }}
    }
}

#[component]
fn StarMapView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;

    // Layout Logic (Memoized)
    let layout = Memo::new(move |_| {
        let s = state.get();
        // 1. Find Hub (Room 0)
        let mut rooms: Vec<Room> = s.map.rooms.values().cloned().collect();
        if rooms.is_empty() {
            return (vec![], vec![], vec![]);
        }

        // Try to find room 0 specifically, or fallback to max neighbors
        let hub = rooms.iter().find(|r| r.id == 0).cloned().or_else(|| {
            rooms.sort_by_key(|r| std::cmp::Reverse(r.neighbors.len()));
            rooms.first().cloned()
        });

        if let Some(hallway) = hub {
            // 2. Split neighbors into Top/Bottom
            let mut top_row = vec![];
            let mut bot_row = vec![];

            // Get all other rooms
            let mut remaining: Vec<Room> =
                rooms.into_iter().filter(|r| r.id != hallway.id).collect();
            remaining.sort_by_key(|r| r.id); // Stable sort

            for (i, room) in remaining.into_iter().enumerate() {
                if i % 2 == 0 {
                    top_row.push(room);
                } else {
                    bot_row.push(room);
                }
            }
            (vec![hallway], top_row, bot_row)
        } else {
            (vec![], vec![], vec![])
        }
    });

    view! {
        <div style="
        display: grid; 
        grid-template-rows: auto auto auto; 
        gap: 15px; 
        background: #111; 
        padding: 20px; 
        border-radius: 12px;
        border: 2px solid #333;
        width: 100%;
        max-width: 100%;
        box-sizing: border-box;
        ">
            // Top Row
            <div style="display: flex; gap: 15px; width: 100%;">
                {({
                    let ctx_top = ctx.clone();
                    move || {
                        layout
                            .get()
                            .1
                            .into_iter()
                            .map(|r| {
                                view! {
                                    <RoomCard
                                        room=r.clone()
                                        ctx=ctx_top.clone()
                                        door_dir=Some(DoorDirection::Bottom)
                                    />
                                }
                            })
                            .collect::<Vec<_>>()
                    }
                })()}
            </div>

            // Hallway (Spine)
            <div style="display: flex; width: 100%;">
                {({
                    let ctx_mid = ctx.clone();
                    move || {
                        layout
                            .get()
                            .0
                            .into_iter()
                            .map(|r| {
                                view! {
                                    <div style="width: 100%; display: flex;">
                                        <RoomCard
                                            room=r.clone()
                                            ctx=ctx_mid.clone()
                                            door_dir=None
                                        />
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }
                })()}
            </div>

            // Bottom Row
            <div style="display: flex; gap: 15px; width: 100%;">
                {({
                    let ctx_bot = ctx.clone();
                    move || {
                        layout
                            .get()
                            .2
                            .into_iter()
                            .map(|r| {
                                view! {
                                    <RoomCard
                                        room=r.clone()
                                        ctx=ctx_bot.clone()
                                        door_dir=Some(DoorDirection::Top)
                                    />
                                }
                            })
                            .collect::<Vec<_>>()
                    }
                })()}
            </div>
        </div>
    }
}

#[component]
fn TorusMapView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;

    // Grid Layout: 4x4 Hollow Square
    // 0  1  2  3
    // 11 .  .  4
    // 10 .  .  5
    // 9  8  7  6

    let grid_map = vec![
        // Row 0
        (0, 0, Some(DoorDirection::Right)),
        (0, 1, Some(DoorDirection::Right)),
        (0, 2, Some(DoorDirection::Right)),
        (0, 3, Some(DoorDirection::Bottom)),
        // Right Col
        (1, 3, Some(DoorDirection::Bottom)),
        (2, 3, Some(DoorDirection::Bottom)),
        // Bottom Row (Reversed)
        (3, 3, Some(DoorDirection::Left)),
        (3, 2, Some(DoorDirection::Left)),
        (3, 1, Some(DoorDirection::Left)),
        (3, 0, Some(DoorDirection::Top)),
        // Left Col (Reversed)
        (2, 0, Some(DoorDirection::Top)),
        (1, 0, Some(DoorDirection::Top)),
    ];

    // Map Room ID (index in vector) to Grid Position
    // Room 0 -> (0,0), Room 1 -> (0,1)...
    // The Torus generator creates IDs 0..11 sequentially.

    let cells = Memo::new(move |_| {
        let s = state.get();
        let mut cell_views = vec![];

        for (room_idx, (row, col, door)) in grid_map.iter().enumerate() {
            let rid = room_idx as u32;
            if let Some(room) = s.map.rooms.get(&rid) {
                cell_views.push((*row, *col, *door, room.clone()));
            }
        }
        cell_views
    });

    view! {
        <div style="
        display: grid; 
        grid-template-rows: repeat(4, 1fr); 
        grid-template-columns: repeat(4, 1fr);
        gap: 10px; 
        background: #111; 
        padding: 20px; 
        border-radius: 12px;
        border: 2px solid #333;
        width: 100%;
        height: 600px;
        box-sizing: border-box;
        ">
            {({
                let ctx_inner = ctx.clone();
                move || {
                    cells
                        .get()
                        .into_iter()
                        .map(|(row, col, door, room)| {
                            let grid_area = format!("{}/{}", row + 1, col + 1);
                            // Grid Area Calculation: row / col are 0-based. CSS Grid is 1-based.
                            view! {
                                <div style=format!(
                                    "grid-area: {}; min-width: 0; min-height: 0;",
                                    grid_area,
                                )>
                                    <RoomCard room=room ctx=ctx_inner.clone() door_dir=door />
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }
            })()}
        </div>
    }
}

#[component]
fn RoomCard(room: Room, ctx: GameContext, door_dir: Option<DoorDirection>) -> impl IntoView {
    let state_sig = ctx.state;
    let my_pid = ctx.player_id.clone();
    let ctx_click = ctx.clone();

    // Computed State
    let my_pid_memo = my_pid.clone();
    let room_memo = room.clone();

    let calc = Memo::new(move |_| {
        let s = state_sig.get();
        let my_pid = &my_pid_memo;
        let room = &room_memo;

        // 1. Players Here
        let players_here: Vec<Player> = s
            .players
            .values()
            .filter(|p| p.room_id == room.id)
            .cloned()
            .collect();

        let is_here = players_here.iter().any(|p| p.id == *my_pid);

        // 2. Prediction (Where will I be?)
        let mut predicted_room_id = 0;
        let mut predicted_ap = 0;

        if let Some(p) = s.players.get(my_pid) {
            predicted_room_id = p.room_id;
            predicted_ap = p.ap;

            // The proposal queue moves are already paid for in p.ap (logic.rs),
            // but we need to track position.
            for prop in &s.proposal_queue {
                if prop.player_id == *my_pid {
                    if let GameAction::Move { to_room } = prop.action {
                        predicted_room_id = to_room;
                    }
                }
            }
        }

        // 3. Pathfinding
        let mut path = None;
        let mut can_move = false;

        if s.phase == GamePhase::TacticalPlanning
            && predicted_room_id != 0
            && predicted_room_id != room.id
        {
            if let Some(p) = find_path(&s.map, predicted_room_id, room.id) {
                let mut cost = 0;
                for step in &p {
                    cost += sint_core::logic::actions::action_cost(
                        &s,
                        my_pid,
                        &GameAction::Move { to_room: *step },
                    );
                }

                if cost <= predicted_ap {
                    can_move = true;
                    path = Some(p);
                }
            }
        }

        // 4. Other Status
        let has_fire = room.hazards.contains(&HazardType::Fire);
        let has_water = room.hazards.contains(&HazardType::Water);
        let is_targeted = s
            .enemy
            .next_attack
            .as_ref()
            .is_some_and(|a| a.target_room == room.id && a.effect != AttackEffect::Miss);

        // 5. Ghosts
        let ghosts: Vec<String> = s
            .proposal_queue
            .iter()
            .filter_map(|prop| {
                if let GameAction::Move { to_room } = prop.action {
                    if to_room == room.id {
                        return Some(prop.player_id.clone());
                    }
                }
                None
            })
            .collect();

        (
            players_here,
            is_here,
            can_move,
            path,
            has_fire,
            has_water,
            is_targeted,
            ghosts,
        )
    });

    // VIEW
    view! {
        {move || {
            let (players_here, is_here, can_move, path, has_fire, has_water, is_targeted, ghosts) = calc
                .get();
            let ctx_click_inner = ctx_click.clone();
            let bg_color = if has_fire {
                "#3e1a1a"
            } else if has_water {
                "#1a2a3e"
            } else {
                "#2a2a2a"
            };
            let border_color = if is_here {
                "#4caf50"
            } else if is_targeted {
                "#f44336"
            } else if can_move {
                "#2196f3"
            } else {
                "#555"
            };
            let border_style = if is_targeted { "dashed" } else { "solid" };
            let min_height = "120px";
            let cursor = if can_move { "pointer" } else { "default" };
            let hover_style = if can_move { "filter: brightness(1.2);" } else { "" };
            let shadow = if can_move { "0 0 8px #2196f3" } else { "0 4px 6px rgba(0,0,0,0.3)" };

            view! {
                <div
                    style=format!(
                        "
                        background: {}; 
                        border: 2px {} {}; 
                        border-radius: 8px; 
                        padding: 10px; 
                        min-height: {};
                        position: relative;
                        transition: all 0.2s;
                        box-shadow: {};
                        flex: 1;
                        min-width: 80px;
                        width: auto;
                        cursor: {};
                        {}",
                        bg_color,
                        border_style,
                        border_color,
                        min_height,
                        shadow,
                        cursor,
                        hover_style,
                    )
                    on:click=move |_| {
                        if can_move {
                            if let Some(steps) = path.clone() {
                                for step in steps {
                                    ctx_click_inner
                                        .perform_action
                                        .call(Action::Game(GameAction::Move { to_room: step }));
                                }
                            }
                        }
                    }
                >

                    // Door Connector
                    {
                        let d_style = "position: absolute; width: 20px; height: 20px; z-index: 10; background: #555; border: 1px solid #333;";
                        match door_dir {
                            Some(DoorDirection::Top) => {
                                Either::Left(

                                    view! {
                                        <div style=format!(
                                            "{}; top: -10px; left: 50%; transform: translateX(-50%);",
                                            d_style,
                                        )></div>
                                    },
                                )
                            }
                            Some(DoorDirection::Bottom) => {
                                Either::Left(
                                    view! {
                                        <div style=format!(
                                            "{}; bottom: -10px; left: 50%; transform: translateX(-50%);",
                                            d_style,
                                        )></div>
                                    },
                                )
                            }
                            Some(DoorDirection::Left) => {
                                Either::Left(
                                    view! {
                                        <div style=format!(
                                            "{}; left: -10px; top: 50%; transform: translateY(-50%);",
                                            d_style,
                                        )></div>
                                    },
                                )
                            }
                            Some(DoorDirection::Right) => {
                                Either::Left(
                                    view! {
                                        <div style=format!(
                                            "{}; right: -10px; top: 50%; transform: translateY(-50%);",
                                            d_style,
                                        )></div>
                                    },
                                )
                            }
                            _ => Either::Right(view! {}),
                        }
                    }

                    // Target Indicator
                    {if is_targeted {
                        Either::Left(
                            view! {
                                <div style="position: absolute; top: -10px; right: -10px; background: #f44336; color: white; padding: 2px 8px; border-radius: 10px; font-weight: bold; font-size: 0.8em; box-shadow: 0 2px 4px rgba(0,0,0,0.5);">
                                    "TARGET"
                                </div>
                            },
                        )
                    } else {
                        Either::Right(view! {})
                    }}

                    // Header
                    <div style="border-bottom: 1px solid #444; padding-bottom: 5px; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
                        <span style="font-weight: bold;">
                            {format!("{} {}", room.id, room.name)}
                        </span>
                        <span style="font-size: 0.7em; color: #888; text-transform: uppercase;">
                            {format!("{:?}", room.system).replace("Some(", "").replace(")", "")}
                        </span>
                    </div>

                    // Content
                    <div style="font-size: 0.9em;">
                        // Hazards
                        <div style="display: flex; gap: 5px; margin-bottom: 5px;">
                            {room
                                .hazards
                                .iter()
                                .map(|h| {
                                    match h {
                                        HazardType::Fire => {
                                            view! { <span title="Fire">"🔥"</span> }
                                        }
                                        HazardType::Water => {
                                            view! { <span title="Water">"💧"</span> }
                                        }
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>

                        // Items
                        <div style="display: flex; gap: 5px; flex-wrap: wrap; margin-bottom: 5px;">
                            {room
                                .items
                                .iter()
                                .map(|i| {
                                    match i {
                                        ItemType::Peppernut => {
                                            view! { <span title="Peppernut">"🍪"</span> }
                                        }
                                        ItemType::Extinguisher => {
                                            view! { <span title="Extinguisher">"🧯"</span> }
                                        }
                                        ItemType::Keychain => {
                                            view! { <span title="Keychain">"🔑"</span> }
                                        }
                                        ItemType::Wheelbarrow => {
                                            view! { <span title="Wheelbarrow">"🛒"</span> }
                                        }
                                        ItemType::Mitre => {
                                            view! { <span title="Mitre">"🧢"</span> }
                                        }
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>

                        // Players
                        <div style="display: flex; flex-direction: column; gap: 2px;">
                            {players_here
                                .into_iter()
                                .map(|p| {
                                    let is_me = p.id == my_pid;
                                    let color = if is_me { "#81c784" } else { "#ddd" };
                                    let fainted = p
                                        .status
                                        .contains(&sint_core::PlayerStatus::Fainted);
                                    let icon = if fainted { "💀" } else { "👤" };
                                    let ready_mark = if p.is_ready { " ✅" } else { "" };

                                    view! {
                                        <div style=format!(
                                            "color: {}; font-size: 0.85em;",
                                            color,
                                        )>{icon} " " {p.name} {ready_mark}</div>
                                    }
                                })
                                .collect::<Vec<_>>()} // Ghosts
                            {ghosts
                                .into_iter()
                                .map(|pid| {
                                    let is_me = pid == my_pid;
                                    let color = if is_me { "#81c784" } else { "#aaa" };

                                    view! {
                                        <div style=format!(
                                            "color: {}; font-size: 0.85em; opacity: 0.6; font-style: italic;",
                                            color,
                                        )>"👻 " {pid} " (Moving)"</div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            }
        }}
    }
}
