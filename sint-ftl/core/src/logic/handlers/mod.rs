use crate::types::{GameAction, GameState};
use crate::GameError;

pub trait ActionHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError>;
    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        simulation: bool,
    ) -> Result<(), GameError>;
    fn base_cost(&self) -> i32 {
        1
    }
}

pub mod hazard_actions;
pub mod item_actions;
pub mod move_action;
pub mod system_actions;

pub fn get_handler(action: &GameAction) -> Box<dyn ActionHandler> {
    match action {
        GameAction::Move { to_room } => Box::new(move_action::MoveHandler { to_room: *to_room }),
        GameAction::Bake => Box::new(system_actions::BakeHandler),
        GameAction::Shoot => Box::new(system_actions::ShootHandler),
        GameAction::RaiseShields => Box::new(system_actions::RaiseShieldsHandler),
        GameAction::EvasiveManeuvers => Box::new(system_actions::EvasiveManeuversHandler),
        GameAction::Lookout => Box::new(system_actions::LookoutHandler),
        GameAction::FirstAid { target_player } => Box::new(system_actions::FirstAidHandler {
            target_player: target_player.clone(),
        }),
        GameAction::PickUp { item_type } => Box::new(item_actions::PickUpHandler {
            item_type: item_type.clone(),
        }),
        GameAction::Drop { item_index } => Box::new(item_actions::DropHandler {
            item_index: *item_index,
        }),
        GameAction::Throw {
            target_player,
            item_index,
        } => Box::new(item_actions::ThrowHandler {
            target_player: target_player.clone(),
            item_index: *item_index,
        }),
        GameAction::Extinguish => Box::new(hazard_actions::ExtinguishHandler),
        GameAction::Repair => Box::new(hazard_actions::RepairHandler),
        GameAction::Revive { target_player } => Box::new(hazard_actions::ReviveHandler {
            target_player: target_player.clone(),
        }),
        GameAction::Interact => Box::new(hazard_actions::InteractHandler),
        _ => Box::new(NoOpHandler),
    }
}

struct NoOpHandler;
impl ActionHandler for NoOpHandler {
    fn validate(&self, _state: &GameState, _player_id: &str) -> Result<(), GameError> {
        Ok(())
    }
    fn execute(
        &self,
        _state: &mut GameState,
        _player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        Ok(())
    }
    fn base_cost(&self) -> i32 {
        0
    }
}
