import gymnasium as gym
import numpy as np
from gymnasium import spaces
import sint_core
import sint_solver
import json
from typing import Optional, List, Dict, Any
import os
import sys

# Add the project root to sys.path to import ai/game_types.py
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
from ai.game_types import GameState, GameAction, Action, GamePhase, ItemType, HazardType, CardId, SystemType

class SintEnv(gym.Env):
    metadata = {"render_modes": ["human"]}

    def __init__(self, num_players=4, seed=12345):
        super(SintEnv, self).__init__()
        self.num_players = num_players
        self.player_ids = [f"P{i+1}" for i in range(num_players)]
        self.initial_seed = seed
        self.state: Optional[Dict[str, Any]] = None
        
        # Action space: 
        self.action_space = spaces.Discrete(41)
        
        # Observation space (simplified)
        self.observation_space = spaces.Box(low=-1000, high=1000000, shape=(185,), dtype=np.float32)

    def _get_obs(self):
        if not self.state:
            return np.zeros(self.observation_space.shape, dtype=np.float32)
        
        s = self.state
        obs = []
        
        # Global (8)
        obs.append(float(s['hull_integrity']))
        obs.append(float(s['turn_count']))
        obs.append(float(s['boss_level']))
        obs.append(float(s['enemy']['hp']))
        obs.append(1.0 if s['evasion_active'] else 0.0)
        obs.append(1.0 if s['shields_active'] else 0.0)
        obs.append(float(list(GamePhase).index(GamePhase(s['phase']))))
        obs.append(1.0 if s['is_resting'] else 0.0)
        
        # Rooms (10 rooms * 9 = 90)
        rooms = s['map']['rooms']
        for i in range(10):
            room = rooms.get(i)
            if room:
                obs.append(float(room['hazards'].count('Fire')))
                obs.append(float(room['hazards'].count('Water')))
                # Items: [Peppernut, Extinguisher, Keychain, Wheelbarrow, Mitre]
                for item_type in ItemType:
                    obs.append(float(room['items'].count(item_type.value)))
                obs.append(1.0 if room['system'] else 0.0)
                if room['system']:
                    obs.append(float(list(SystemType).index(SystemType(room['system']))))
                else:
                    obs.append(-1.0)
            else:
                obs.extend([0.0] * 9)
        
        # Players (4 players * 9 = 36)
        active_player_id = self._get_active_player_id()
        players = s['players']
        for i in range(4):
            pid = f"P{i+1}"
            p = players.get(pid)
            if p:
                obs.append(float(p['room_id']))
                obs.append(float(p['hp']))
                obs.append(float(p['ap']))
                # Inventory: [Peppernut, Extinguisher, Keychain, Wheelbarrow, Mitre]
                for item_type in ItemType:
                    obs.append(float(p['inventory'].count(item_type.value)))
                obs.append(1.0 if pid == active_player_id else 0.0)
            else:
                obs.extend([0.0] * 9)
        
        # Active Situations (49) - Multi-hot
        all_cards = list(CardId)
        situation_ids = [c['id'] for c in s['active_situations']]
        for card_id in all_cards:
            obs.append(1.0 if card_id.value in situation_ids else 0.0)
            
        # Enemy Intent (2)
        if s['enemy']['next_attack']:
            target_room = s['enemy']['next_attack']['target_room']
            obs.append(float(target_room) if target_room is not None else -1.0)
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
        if seed is None:
            seed = self.initial_seed
        
        # Initialize game
        self.state = sint_core.new_game(self.player_ids, seed)
        
        # Stabilize state (advance to TacticalPlanning if needed)
        self._stabilize()
        
        return self._get_obs(), {}

    def _stabilize(self):
        limit = 100
        while limit > 0:
            limit -= 1
            if self.state['phase'] in ['Victory', 'GameOver']:
                break
            
            active_id = self._get_active_player_id()
            if self.state['phase'] == 'TacticalPlanning' and active_id:
                break
            
            # Auto-advance
            unready = [pid for pid, p in self.state['players'].items() if not p['is_ready']]
            if not unready:
                break
                
            if self.state['phase'] == 'TacticalPlanning':
                for pid in unready:
                    if self.state['players'][pid]['ap'] <= 0:
                        self._apply_raw_action(pid, {"type": "VoteReady", "payload": {"ready": True}})
                        break
                else:
                    break
            else:
                pid = unready[0]
                self._apply_raw_action(pid, {"type": "VoteReady", "payload": {"ready": True}})
                
    def _apply_raw_action(self, player_id, action_dict):
        try:
            self.state = sint_core.apply_action_with_id(self.state, player_id, action_dict, None)
            return True
        except Exception:
            return False

    def step(self, action_idx):
        active_id = self._get_active_player_id()
        if not active_id:
            return self._get_obs(), 0.0, True, False, {}
            
        game_action = self._map_action(action_idx, active_id)
        success = self._apply_raw_action(active_id, game_action)
        
        if not success:
            reward = -10.0
        else:
            self._stabilize()
            try:
                reward = sint_solver.compute_score_rhea(self.state)
                reward = reward / 1000.0
            except Exception:
                reward = 0.0
        
        done = self.state['phase'] in ['Victory', 'GameOver']
        return self._get_obs(), float(reward), done, False, {}

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
        elif 27 <= idx <= 30:
            target_id = f"P{int(idx - 27 + 1)}"
            return {"type": "Throw", "payload": {"item_index": 0, "target_player": target_id}}
        elif 31 <= idx <= 34:
            target_id = f"P{int(idx - 31 + 1)}"
            return {"type": "FirstAid", "payload": {"target_player": target_id}}
        elif 35 <= idx <= 38:
            target_id = f"P{int(idx - 35 + 1)}"
            return {"type": "Revive", "payload": {"target_player": target_id}}
        elif idx == 39:
            return {"type": "VoteReady", "payload": {"ready": True}}
        elif idx == 40:
            return {"type": "Pass"}
        return {"type": "Pass"}

    def action_masks(self):
        # For MaskablePPO
        active_id = self._get_active_player_id()
        if not active_id:
            return np.zeros(41, dtype=bool)
            
        masks = np.zeros(41, dtype=bool)
        valid_actions = sint_core.get_valid_actions(self.state, active_id)
        
        # valid_actions is a list of dicts like [{"type": "Move", "payload": {...}}, ...]
        valid_types = [a['type'] for a in valid_actions]
        
        for i in range(41):
            act = self._map_action(i, active_id)
            if act["type"] in valid_types:
                masks[i] = True
        
        return masks
