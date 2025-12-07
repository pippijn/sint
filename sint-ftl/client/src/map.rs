use leptos::*;
use crate::state::GameContext;
use sint_core::{GameState, RoomId, Room, Player, ItemType, HazardType};
use std::collections::HashMap;

#[component]
pub fn MapView(ctx: GameContext) -> impl IntoView {
    let state = ctx.state;
    let pid = ctx.player_id.clone();
    
    // Simple list view for now, grouped by ID
    view! {
        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 10px;">
            {move || {
                let s = state.get();
                let mut rooms: Vec<&Room> = s.map.rooms.values().collect();
                rooms.sort_by_key(|r| r.id);
                
                rooms.into_iter().map(|room| {
                    view! {
                        <RoomCard 
                            room=room.clone() 
                            state=s.clone() 
                            my_pid=pid.clone() 
                        />
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}

#[component]
fn RoomCard(room: Room, state: GameState, my_pid: String) -> impl IntoView {
    let players_here: Vec<Player> = state.players.values()
        .filter(|p| p.room_id == room.id)
        .cloned()
        .collect();
        
    let is_here = players_here.iter().any(|p| p.id == my_pid);
    
    let bg_color = if is_here { "#444" } else { "#2a2a2a" };
    let border = if is_here { "2px solid #4caf50" } else { "1px solid #555" };

    view! {
        <div style=format!("background: {}; border: {}; padding: 10px; border-radius: 8px;", bg_color, border)>
            <div style="font-weight: bold; border-bottom: 1px solid #555; padding-bottom: 5px; margin-bottom: 5px;">
                {format!("{} - {}", room.id, room.name)}
                <span style="float: right; font-size: 0.8em; color: #aaa;">
                    {format!("{:?}", room.system)}
                </span>
            </div>
            
            // Hazards
            {if !room.hazards.is_empty() {
                view! {
                    <div style="color: #ff5252; margin-bottom: 5px;">
                        "⚠ " {format!("{:?}", room.hazards)}
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
            
            // Items
            {if !room.items.is_empty() {
                 view! {
                    <div style="color: #ffeb3b; font-size: 0.9em; margin-bottom: 5px;">
                        "📦 " {format!("{:?}", room.items)}
                    </div>
                 }.into_view()
            } else {
                 view! {}.into_view()
            }}
            
            // Players
            <div style="margin-top: 5px;">
                {players_here.into_iter().map(|p| {
                    let style = if p.id == my_pid { "color: #4caf50; font-weight: bold;" } else { "color: #ddd;" };
                    view! {
                        <div style=style>
                            "👤 " {p.name} {format!(" ({}/{} HP)", p.hp, 3)}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            
            // Connections (Neighbors)
            <div style="margin-top: 8px; font-size: 0.8em; color: #888;">
                "Doors: " {format!("{:?}", room.neighbors)}
            </div>
        </div>
    }
}
