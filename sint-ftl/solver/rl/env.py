import gymnasium as gym
import numpy as np
from gymnasium import spaces
import sint_core
import sint_solver
import json
import uuid
from typing import Optional, List, Dict, Any, Tuple
import os
import sys

# Add the project root to sys.path to import ai/game_types.py
project_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.append(project_root)
sys.path.append(os.path.join(project_root, "ai"))

from ai.game_types import GameState, GameAction, Action, GamePhase, ItemType, HazardType, CardId, SystemType

class SintEnv(gym.Env):
    metadata = {"render_modes": ["human"]}

    def __init__(self, num_players=6, seed=12345, test_mode=False):
        super(SintEnv, self).__init__()
        self.num_players = num_players
        self.player_ids = [f"P{i+1}" for i in range(num_players)]
        self.initial_seed = seed
        self.session_id = str(uuid.uuid4())
        self.test_mode = test_mode
        self.state: Optional[Dict[str, Any]] = None
        self.history: List[Tuple[str, Any]] = []
        self.last_score = 0.0
        
        # Action space: 
        self.action_space = spaces.Discrete(46)
        
        # Observation space (expanded)
        # Global (8)
        # Rooms (10 * 9 = 90)
        # Players (6 * 9 = 54)
        # Active Situations Multi-hot (49)
        # Situation Details (49 * 5 = 245)
        # Enemy Intent (2)
        # Total: 8 + 90 + 54 + 49 + 245 + 2 = 448
        self.observation_space = spaces.Box(low=-1000, high=1000000, shape=(448,), dtype=np.float32)

    def _get_obs(self):
        if not self.state:
            return np.zeros(self.observation_space.shape, dtype=np.float32)
        
        s = self.state
        obs = []
        
        # Global (8)
        obs.append(float(s['hull_integrity']) / 20.0)
        obs.append(float(s['turn_count']) / 100.0)
        obs.append(float(s['boss_level']) / 10.0)
        obs.append(float(s['enemy']['hp']) / 100.0)
        obs.append(1.0 if s['evasion_active'] else 0.0)
        obs.append(1.0 if s['shields_active'] else 0.0)
        obs.append(float(list(GamePhase).index(GamePhase(s['phase']))) / 10.0)
        obs.append(1.0 if s['is_resting'] else 0.0)
        
        # Rooms (10 rooms * 9 = 90)
        rooms = s['map']['rooms']
        for i in range(10):
            room = rooms.get(str(i)) if str(i) in rooms else rooms.get(i)
            if room:
                obs.append(float(room['hazards'].count('Fire')) / 5.0)
                obs.append(float(room['hazards'].count('Water')) / 5.0)
                # Items: [Peppernut, Extinguisher, Keychain, Wheelbarrow, Mitre]
                for item_type in ItemType:
                    count = room['items'].count(item_type.value)
                    obs.append(float(count) / 10.0)
                obs.append(1.0 if room['system'] else 0.0)
                if room['system']:
                    obs.append(float(list(SystemType).index(SystemType(room['system']))) / 10.0)
                else:
                    obs.append(-1.0)
            else:
                obs.extend([0.0] * 9)
        
        # Players (6 players * 9 = 54)
        active_player_id = self._get_active_player_id()
        players = s['players']
        for i in range(6):
            pid = f"P{i+1}"
            p = players.get(pid)
            if p:
                obs.append(float(p['room_id']) / 10.0)
                obs.append(float(p['hp']) / 3.0)
                obs.append(float(p['ap']) / 4.0)
                # Inventory: [Peppernut, Extinguisher, Keychain, Wheelbarrow, Mitre]
                for item_type in ItemType:
                    count = p['inventory'].count(item_type.value)
                    obs.append(float(count) / 5.0)
                obs.append(1.0 if pid == active_player_id else 0.0)
            else:
                obs.extend([0.0] * 9)
        
        # Active Situations (49) - Multi-hot
        all_cards = list(CardId)
        situation_ids = [c['id'] for c in s['active_situations']]
        situation_map = {c['id']: c for c in s['active_situations']}
        
        for card_id in all_cards:
            obs.append(1.0 if card_id.value in situation_ids else 0.0)
            
        # Situation Details (49 * 5 = 245)
        for card_id in all_cards:
            if card_id.value in situation_map:
                c = situation_map[card_id.value]
                sol = c.get('solution')
                if sol:
                    obs.append(float(sol['ap_cost']) / 10.0)
                    if sol['item_cost']:
                        obs.append(float(list(ItemType).index(ItemType(sol['item_cost']))) / 5.0)
                    else:
                        obs.append(-1.0)
                    if sol['target_system']:
                        obs.append(float(list(SystemType).index(SystemType(sol['target_system']))) / 10.0)
                    else:
                        obs.append(-1.0)
                    obs.append(float(sol['required_players']) / 6.0)
                    # Plus one extra for card-specific logic if needed
                    obs.append(0.0)
                else:
                    obs.extend([0.0] * 5)
            else:
                obs.extend([0.0] * 5)

        # Enemy Intent (2)
        if s['enemy']['next_attack']:
            target_room = s['enemy']['next_attack']['target_room']
            obs.append(float(target_room) / 10.0 if target_room is not None else -1.0)
            obs.append(1.0)
        else:
            obs.extend([-1.0, 0.0])
            
        return np.array(obs, dtype=np.float32)

    def _get_active_player_id(self):
        if not self.state: return None
        players = self.state['players']
        for pid in self.player_ids:
            p = players.get(pid)
            if p and not p['is_ready'] and p['ap'] > 0:
                return pid
        return None

    def reset(self, seed=None, options=None):
        super().reset(seed=seed)
        if seed is not None:
            self.initial_seed = seed
        elif not self.test_mode:
            self.initial_seed = np.random.randint(0, 1000000)
        
        self.history = []
        
        # Use verify_linear to get initial state
        result = sint_solver.verify_linear(self.player_ids, self.initial_seed, self.history)
        self.state = result['final_state']
        assert self.state is not None, "State should not be None after reset"
        self.last_score = result['rl_score']
        
        obs = self._get_obs()
        assert obs.shape == self.observation_space.shape, f"Obs shape {obs.shape} != {self.observation_space.shape}"
        return obs, {}

    def step(self, action_idx):
        active_id = self._get_active_player_id()
        if not active_id:
            return self._get_obs(), 0.0, True, False, {}
        
        # Assertion: Ensure the chosen action was actually valid according to our masks
        # This catches bugs where the agent (or a random sampler) ignores the mask.
        masks = self.action_masks()
        assert masks[action_idx], f"Action {action_idx} was chosen but it is MASKED as invalid for {active_id}"
            
        game_action = self._map_action(action_idx, active_id)
        
        # We only need to verify the LATEST action against the CURRENT state
        latest_action = [(active_id, game_action)]
        
        # Verify incrementally
        result = sint_solver.verify_linear(self.player_ids, self.initial_seed, latest_action, initial_state=self.state)
        
        # Record the action in a linear history
        self.history.append((active_id, game_action))
        
        # Update state
        self.state = result['final_state']
        assert self.state is not None, "State should not be None after step"

        # Reward is the delta in dense rl_score
        current_score = result['rl_score']
        reward = current_score # rl_score in solver/src/scoring/rl.rs is already a delta-based score or immediate reward
        self.last_score = current_score
        done = self.state['phase'] in ['Victory', 'GameOver']
            
        obs = self._get_obs()
        assert isinstance(reward, (float, int, np.float32)), f"Reward should be numeric, got {type(reward)}"
        return obs, float(reward), done, False, {}

    def _is_recoverable(self, error):
        # Errors that just mean "round in progress" or "round finished" are fine for RL
        err_str = str(error)
        return "still have AP" in err_str or "Round advanced" in err_str

    def _actions_equal(self, a1, a2):
        if a1['type'] != a2['type']:
            return False
        p1 = a1.get('payload')
        p2 = a2.get('payload')
        if p1 == p2:
            return True
        if p1 and p2:
            # Handle potential type mismatches (e.g. string vs int room_id)
            return str(p1) == str(p2)
        return False

    def _map_action(self, idx, player_id):
        if idx < 10:
            return {"type": "Move", "payload": {"to_room": int(idx)}}
        elif idx == 10:
            return {"type": "Interact"}
        elif idx == 11:
            return {"type": "Bake"}
        elif idx == 12:
            return {"type": "Shoot"}
        elif idx == 13:
            return {"type": "RaiseShields"}
        elif idx == 14:
            return {"type": "EvasiveManeuvers"}
        elif idx == 15:
            return {"type": "Extinguish"}
        elif idx == 16:
            return {"type": "Repair"}
        elif 17 <= idx <= 21:
            item_type = list(ItemType)[idx - 17]
            return {"type": "PickUp", "payload": {"item_type": item_type.value}}
        elif 22 <= idx <= 26:
            return {"type": "Drop", "payload": {"item_index": int(idx - 22)}}
        elif 27 <= idx <= 32:
            target_id = f"P{int(idx - 27 + 1)}"
            return {"type": "Throw", "payload": {"item_index": 0, "target_player": target_id}}
        elif 33 <= idx <= 38:
            target_id = f"P{int(idx - 33 + 1)}"
            return {"type": "FirstAid", "payload": {"target_player": target_id}}
        elif 39 <= idx <= 44:
            target_id = f"P{int(idx - 39 + 1)}"
            return {"type": "Revive", "payload": {"target_player": target_id}}
        elif idx == 45:
            return {"type": "Pass"}
        return {"type": "Pass"}

    def action_masks(self):
        # For MaskablePPO
        active_id = self._get_active_player_id()
        if not active_id:
            return np.zeros(46, dtype=bool)

        masks = np.zeros(46, dtype=bool)
        valid_actions = sint_core.get_valid_actions(self.state, active_id)

        for i in range(46):
            act = self._map_action(i, active_id)
            if any(self._actions_equal(act, va) for va in valid_actions):
                masks[i] = True

        return masks
            