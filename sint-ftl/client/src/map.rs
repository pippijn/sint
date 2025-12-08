use leptos::*;
use crate::state::GameContext;
use sint_core::{Room, Player, ItemType, HazardType, Action, GameMap, RoomId};
use std::collections::{VecDeque, HashSet};

#[derive(Clone, Copy, PartialEq)]
pub enum DoorDirection {
    Top,
    Bottom,
    None
}

#[component]
pub fn MapView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    
    // Layout Logic (Memoized)
    let layout = create_memo(move |_| {
        let s = state.get();
        // 1. Find Hallway (Node with most neighbors)
        let mut rooms: Vec<Room> = s.map.rooms.values().cloned().collect();
        if rooms.is_empty() { return (vec![], vec![], vec![]); }
        
        rooms.sort_by_key(|r| std::cmp::Reverse(r.neighbors.len()));
        let hallway = rooms[0].clone();
        
        // 2. Split neighbors into Top/Bottom
        let mut top_row = vec![];
        let mut bot_row = vec![];
        
        let mut remaining: Vec<Room> = rooms.into_iter()
            .filter(|r| r.id != hallway.id)
            .collect();
            
        remaining.sort_by_key(|r| r.id); // Stable sort for consistent layout
        
        for (i, room) in remaining.into_iter().enumerate() {
            if i % 2 == 0 {
                top_row.push(room);
            } else {
                bot_row.push(room);
            }
        }
        
        (vec![hallway], top_row, bot_row)
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
                {{
                    let ctx_top = ctx.clone();
                    move || layout.get().1.into_iter().map(|r| {
                         view! { <RoomCard room=r.clone() ctx=ctx_top.clone() door_dir=DoorDirection::Bottom /> }
                    }).collect::<Vec<_>>()
                }}
            </div>
            
            // Hallway (Spine)
            <div style="display: flex; width: 100%;">
                {{
                    let ctx_mid = ctx.clone();
                    move || layout.get().0.into_iter().map(|r| {
                        view! { 
                            <div style="width: 100%; display: flex;">
                                <RoomCard room=r.clone() ctx=ctx_mid.clone() /> 
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
            
            // Bottom Row
            <div style="display: flex; gap: 15px; width: 100%;">
                {{
                    let ctx_bot = ctx.clone();
                    move || layout.get().2.into_iter().map(|r| {
                         view! { <RoomCard room=r.clone() ctx=ctx_bot.clone() door_dir=DoorDirection::Top /> }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

#[component]
fn RoomCard(
    room: Room, 
    ctx: GameContext,
    #[prop(optional)]
    door_dir: Option<DoorDirection>,
) -> impl IntoView {
    let state_sig = ctx.state;
    let my_pid = ctx.player_id.clone();
    let ctx_click = ctx.clone();
    
    // Computed State
    let my_pid_memo = my_pid.clone();
    let room_memo = room.clone();
    
    let calc = create_memo(move |_| {
        let s = state_sig.get();
        let my_pid = &my_pid_memo;
        let room = &room_memo;
        
        // 1. Players Here
        let players_here: Vec<Player> = s.players.values()
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
                      if let Action::Move { to_room } = prop.action {
                          predicted_room_id = to_room;
                      }
                 }
             }
        }
        
        // 3. Pathfinding
        let mut path = None;
        let mut can_move = false;
        
        if predicted_room_id != 0 && predicted_room_id != room.id {
            if let Some(p) = find_path(&s.map, predicted_room_id, room.id) {
                let slippery = s.active_situations.iter().any(|c| c.id == "C04");
                let cost = if slippery { 0 } else { p.len() as i32 };
                
                if cost <= predicted_ap {
                    can_move = true;
                    path = Some(p);
                }
            }
        }
        
        // 4. Other Status
        let has_fire = room.hazards.contains(&HazardType::Fire);
        let has_water = room.hazards.contains(&HazardType::Water);
        let is_targeted = s.enemy.next_attack.as_ref().map_or(false, |a| a.target_room == room.id);
        
        // 5. Ghosts
        let ghosts: Vec<String> = s.proposal_queue.iter()
            .filter_map(|prop| {
                if let Action::Move { to_room } = prop.action {
                    if to_room == room.id {
                        return Some(prop.player_id.clone());
                    }
                }
                None
            })
            .collect();

        (players_here, is_here, can_move, path, has_fire, has_water, is_targeted, ghosts)
    });

    // VIEW
    view! {
        {move || {
            let (players_here, is_here, can_move, path, has_fire, has_water, is_targeted, ghosts) = calc.get();
            let ctx_click_inner = ctx_click.clone();
            
            let bg_color = if has_fire { "#3e1a1a" } else if has_water { "#1a2a3e" } else { "#2a2a2a" };
            let border_color = if is_here { "#4caf50" } else if is_targeted { "#f44336" } else if can_move { "#2196f3" } else { "#555" };
            let border_style = if is_targeted { "dashed" } else { "solid" };
            let min_height = "120px";
            let cursor = if can_move { "pointer" } else { "default" };
            let hover_style = if can_move { "filter: brightness(1.2);" } else { "" };
            let shadow = if can_move { "0 0 8px #2196f3" } else { "0 4px 6px rgba(0,0,0,0.3)" };

            view! {
                <div 
                    style=format!("
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
                        {}", bg_color, border_style, border_color, min_height, shadow, cursor, hover_style)
                    on:click=move |_| {
                        if can_move {
                             if let Some(steps) = path.clone() {
                                 for step in steps {
                                     ctx_click_inner.perform_action.call(Action::Move { to_room: step });
                                 }
                             }
                        }
                    }
                >
                    
                    // Door Connector
                    {
                        let d_style = "position: absolute; left: 50%; transform: translateX(-50%); width: 40px; height: 10px; z-index: 10;";
                        let d_border = format!("2px {} {}", border_style, border_color);
                        
                        match door_dir {
                            Some(DoorDirection::Top) => view! {
                                <div style=format!("{}; background: {}; top: -12px; border: {}; border-bottom: none; border-radius: 4px 4px 0 0;", d_style, bg_color, d_border)></div>
                            }.into_view(),
                            Some(DoorDirection::Bottom) => view! {
                                 <div style=format!("{}; background: {}; bottom: -12px; border: {}; border-top: none; border-radius: 0 0 4px 4px;", d_style, bg_color, d_border)></div>
                            }.into_view(),
                            _ => view! {}.into_view()
                        }
                    }

                    // Target Indicator
                    {if is_targeted {
                        view! {
                            <div style="position: absolute; top: -10px; right: -10px; background: #f44336; color: white; padding: 2px 8px; border-radius: 10px; font-weight: bold; font-size: 0.8em; box-shadow: 0 2px 4px rgba(0,0,0,0.5);">
                                "TARGET"
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}

                    // Header
                    <div style="border-bottom: 1px solid #444; padding-bottom: 5px; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
                        <span style="font-weight: bold;">{format!("{} {}", room.id, room.name)}</span>
                        <span style="font-size: 0.7em; color: #888; text-transform: uppercase;">
                            {format!("{:?}", room.system).replace("Some(", "").replace(")", "")}
                        </span>
                    </div>
                    
                    // Content
                    <div style="font-size: 0.9em;">
                        // Hazards
                        <div style="display: flex; gap: 5px; margin-bottom: 5px;">
                            {room.hazards.iter().map(|h| {
                                match h {
                                    HazardType::Fire => view! { <span title="Fire">"🔥"</span> },
                                    HazardType::Water => view! { <span title="Water">"💧"</span> },
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        
                        // Items
                        <div style="display: flex; gap: 5px; flex-wrap: wrap; margin-bottom: 5px;">
                            {room.items.iter().map(|i| {
                                match i {
                                    ItemType::Peppernut => view! { <span title="Peppernut">"🍪"</span> },
                                    _ => view! { <span title="Item">"📦"</span> },
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        
                        // Players
                        <div style="display: flex; flex-direction: column; gap: 2px;">
                            {players_here.into_iter().map(|p| {
                                let is_me = p.id == my_pid;
                                let color = if is_me { "#81c784" } else { "#ddd" };
                                let fainted = p.status.contains(&sint_core::PlayerStatus::Fainted);
                                let icon = if fainted { "💀" } else { "👤" };
                                
                                view! {
                                    <div style=format!("color: {}; font-size: 0.85em;", color)>
                                        {icon} " " {p.name}
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                            
                            // Ghosts
                            {ghosts.into_iter().map(|pid| {
                                let is_me = pid == my_pid;
                                let color = if is_me { "#81c784" } else { "#aaa" };
                                
                                view! {
                                    <div style=format!("color: {}; font-size: 0.85em; opacity: 0.6; font-style: italic;", color)>
                                        "👻 " {pid} " (Moving)"
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            }
        }}
    }
}

fn find_path(map: &GameMap, start: RoomId, end: RoomId) -> Option<Vec<RoomId>> {
    if start == end { return Some(vec![]); }
    
    let mut queue = VecDeque::new();
    queue.push_back(vec![start]);
    
    let mut visited = HashSet::new();
    visited.insert(start);
    
    while let Some(path) = queue.pop_front() {
        let last = *path.last().unwrap();
        if last == end {
            return Some(path.into_iter().skip(1).collect());
        }
        
        if path.len() > 3 { continue; } 

        if let Some(room) = map.rooms.get(&last) {
            for &neighbor in &room.neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back(new_path);
                }
            }
        }
    }
    None
}