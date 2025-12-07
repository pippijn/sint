use leptos::*;
use crate::state::GameContext;
use sint_core::{GameState, RoomId, Room, Player, ItemType, HazardType};
use std::collections::HashMap;

#[component]
pub fn MapView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    
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

    let pid_top = pid.clone();
    let pid_mid = pid.clone();
    let pid_bot = pid.clone();

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
                {move || layout.get().1.into_iter().map(|r| {
                    view! { <RoomCard room=r.clone() state=state.get() my_pid=pid_top.clone() /> }
                }).collect::<Vec<_>>()}
            </div>
            
            // Hallway (Spine)
            <div style="display: flex; width: 100%;">
                {move || layout.get().0.into_iter().map(|r| {
                    view! { 
                        <div style="width: 100%; display: flex;">
                            <RoomCard room=r.clone() state=state.get() my_pid=pid_mid.clone() is_hallway=true /> 
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            
            // Bottom Row
            <div style="display: flex; gap: 15px; width: 100%;">
                {move || layout.get().2.into_iter().map(|r| {
                    view! { <RoomCard room=r.clone() state=state.get() my_pid=pid_bot.clone() /> }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn RoomCard(
    room: Room, 
    state: GameState, 
    my_pid: String,
    #[prop(optional)] is_hallway: bool
) -> impl IntoView {
    let players_here: Vec<Player> = state.players.values()
        .filter(|p| p.room_id == room.id)
        .cloned()
        .collect();
        
    let is_here = players_here.iter().any(|p| p.id == my_pid);
    
    // Check Hazards
    let has_fire = room.hazards.contains(&HazardType::Fire);
    let has_water = room.hazards.contains(&HazardType::Water);
    
    // Check if Targeted
    let is_targeted = state.enemy.next_attack.as_ref().map_or(false, |a| a.target_room == room.id);

    // Styling
    let bg_color = if has_fire { "#3e1a1a" } else if has_water { "#1a2a3e" } else { "#2a2a2a" };
    let border_color = if is_here { "#4caf50" } else if is_targeted { "#f44336" } else { "#555" };
    let border_style = if is_targeted { "dashed" } else { "solid" };
    let min_height = "120px";

    view! {
        <div style=format!("
            background: {}; 
            border: 2px {} {}; 
            border-radius: 8px; 
            padding: 10px; 
            min-height: {};
            position: relative;
            transition: all 0.2s;
            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
            flex: 1;
            min-width: 80px;
            width: auto;
        ", bg_color, border_style, border_color, min_height)>
            
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
                </div>
            </div>
        </div>
    }
}
