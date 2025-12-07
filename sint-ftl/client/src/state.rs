use leptos::*;
use sint_core::{GameState, Action, ProposedAction, GameLogic};
use std::collections::VecDeque;
use uuid::Uuid;
use crate::ws::{ClientMessage, ServerMessage};
use gloo_net::websocket::{futures::WebSocket, Message};
use futures::{StreamExt, SinkExt};
use futures::channel::mpsc;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct GameContext {
    pub state: ReadSignal<GameState>,
    pub player_id: String,
    pub room_id: String,
    pub perform_action: ActionCallback,
}

#[derive(Clone)]
pub struct ActionCallback(Rc<dyn Fn(Action)>);

impl ActionCallback {
    pub fn call(&self, action: Action) {
        (self.0)(action)
    }
}

pub fn provide_game_context() -> GameContext {
    let player_id = "Player_1".to_string(); 
    let room_id = "Room_A".to_string();

    let initial_state = GameLogic::new_game(vec![player_id.clone()], 12345);
    let (state, set_state) = create_signal(initial_state.clone());

    // Channel for sending messages to WebSocket
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Internal State for Rollback Logic
    struct InternalState {
        verified_state: GameState,
        pending_actions: VecDeque<ProposedAction>,
    }
    
    let internal = Rc::new(RefCell::new(InternalState {
        verified_state: initial_state.clone(),
        pending_actions: VecDeque::new(),
    }));

    // Spawn WebSocket Task
    let internal_ws = internal.clone();
    let pid_ws = player_id.clone();
    let rid_ws = room_id.clone();
    let set_state_ws = set_state;

    spawn_local(async move {
        let url = "ws://localhost:3000/ws";
        let ws = match WebSocket::open(url) {
            Ok(ws) => ws,
            Err(e) => {
                leptos::logging::error!("Failed to connect: {:?}", e);
                return;
            }
        };

        let (mut write, read) = ws.split();
        let mut read = read.fuse(); // Enable select! macro usage

        // Send Join immediately
        let join_msg = ClientMessage::Join {
            room_id: rid_ws,
            player_id: pid_ws,
        };
        let _ = write.send(Message::Text(serde_json::to_string(&join_msg).unwrap())).await;

        loop {
            futures::select! {
                // Outgoing (from Client UI)
                msg = rx.next() => {
                    if let Some(text) = msg {
                        let _ = write.send(Message::Text(text)).await;
                    }
                },
                // Incoming (from Server)
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                             match serde_json::from_str::<ServerMessage>(&text) {
                                Ok(ServerMessage::Event { sequence_id, data }) => {
                                    let mut guard = internal_ws.borrow_mut();
                                    
                                    if let Ok(proposed) = serde_json::from_value::<ProposedAction>(data) {
                                        leptos::logging::log!("Recv Seq: {}", sequence_id);

                                        // 1. Apply to Verified
                                        let res = GameLogic::apply_action(
                                            guard.verified_state.clone(),
                                            &proposed.player_id,
                                            proposed.action,
                                            None
                                        );

                                        match res {
                                            Ok(new_state) => {
                                                guard.verified_state = new_state;
                                                
                                                // 2. Prune Pending (Match UUID)
                                                if let Some(front) = guard.pending_actions.front() {
                                                    if front.id == proposed.id {
                                                        guard.pending_actions.pop_front();
                                                    }
                                                }

                                                // 3. Replay Pending
                                                let mut predicted = guard.verified_state.clone();
                                                let mut valid_pending = VecDeque::new();
                                                
                                                for p in guard.pending_actions.iter() {
                                                    if let Ok(next) = GameLogic::apply_action(
                                                        predicted.clone(),
                                                        &p.player_id,
                                                        p.action.clone(),
                                                        None
                                                    ) {
                                                        predicted = next;
                                                        valid_pending.push_back(p.clone());
                                                    } else {
                                                        leptos::logging::warn!("Replay invalid: {:?}", p.action);
                                                    }
                                                }
                                                guard.pending_actions = valid_pending;
                                                set_state_ws.set(predicted);
                                            }
                                            Err(e) => {
                                                leptos::logging::error!("Sync Error: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                Ok(ServerMessage::Welcome { room_id }) => {
                                    leptos::logging::log!("Welcome to {}", room_id);
                                }
                                _ => {}
                             }
                        }
                        None => {
                            leptos::logging::warn!("WS Closed");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    // Action Callback
    let internal_action = internal.clone();
    let pid_action = player_id.clone();
    let tx_cell = RefCell::new(tx); // Wrap sender in RefCell
    
    let perform_action = ActionCallback(Rc::new(move |action: Action| {
        let mut guard = internal_action.borrow_mut();
        
        // 1. Optimistic Apply to CURRENT Predicted State
        let mut current_predicted = guard.verified_state.clone();
        for p in &guard.pending_actions {
            current_predicted = GameLogic::apply_action(current_predicted, &p.player_id, p.action.clone(), None).unwrap();
        }
        
        // Now try new action
        match GameLogic::apply_action(
            current_predicted.clone(),
            &pid_action,
            action.clone(),
            None
        ) {
            Ok(new_predicted) => {
                // Success
                set_state.set(new_predicted);
                
                let proposal = ProposedAction {
                    id: Uuid::new_v4().to_string(),
                    player_id: pid_action.clone(),
                    action: action.clone(),
                };
                
                guard.pending_actions.push_back(proposal.clone());
                
                let msg = ClientMessage::Event {
                    sequence_id: 0,
                    data: serde_json::to_value(&proposal).unwrap(),
                };
                
                let _ = tx_cell.borrow_mut().try_send(serde_json::to_string(&msg).unwrap());
            }
            Err(e) => {
                leptos::logging::warn!("Invalid Action: {:?}", e);
            }
        }
    }));

    GameContext {
        state,
        player_id,
        room_id,
        perform_action,
    }
}