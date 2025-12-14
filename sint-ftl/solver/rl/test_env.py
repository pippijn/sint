import unittest
import numpy as np
import os
import sys

# Add current directory to path to import env
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from env import SintEnv
import sint_solver

class TestSintEnv(unittest.TestCase):
    def setUp(self):
        # Use 4 players for faster tests
        self.env = SintEnv(num_players=4, seed=12345, test_mode=True)

    def test_reset(self):
        obs, info = self.env.reset()
        self.assertEqual(obs.shape, (448,))
        self.assertIn(self.env.state['phase'], ['TacticalPlanning', 'MorningReport', 'EnemyTelegraph'])
        # verify_linear should have advanced to TacticalPlanning
        self.assertEqual(self.env.state['phase'], 'TacticalPlanning')
        self.assertEqual(self.env.state['turn_count'], 1)

    def test_action_masks(self):
        self.env.reset()
        masks = self.env.action_masks()
        self.assertEqual(len(masks), 46)
        self.assertTrue(masks.any(), "At least one action should be valid")
        # Pass (45) should be valid initially
        self.assertTrue(masks[45], "Pass should be valid for a player with AP")

    def test_step_and_reward(self):
        self.env.reset()
        initial_score = self.env.last_score
        
        # Perform a 'Pass' action (index 45)
        obs, reward, done, truncated, info = self.env.step(45)
        
        self.assertIsInstance(reward, float)
        self.assertFalse(done)
        # Score might change even on Pass due to station keeping or other passive heuristics in beam_score
        # but here we just check it doesn't crash and returns a numeric reward.
        self.assertEqual(obs.shape, (448,))

    def test_round_advancement(self):
        self.env.reset()
        initial_round = self.env.state['turn_count']
        
        # Clear all AP for all players to force round advancement
        # 4 players * 2 AP = 8 actions needed
        for i in range(12): 
            masks = self.env.action_masks()
            if not masks[45]:
                break
            obs, reward, done, truncated, info = self.env.step(45)
            if done:
                break
        
        self.assertGreater(self.env.state['turn_count'], initial_round, 
                          f"Round should have advanced. Current round: {self.env.state['turn_count']}")
        self.assertEqual(self.env.state['phase'], 'TacticalPlanning')

    def test_incremental_vs_full_verification(self):
        # Reset with fixed seed
        self.env.reset(seed=12345)
        
        actions = []
        # Take some random valid steps
        for _ in range(5):
            masks = self.env.action_masks()
            valid_indices = np.where(masks)[0]
            action_idx = int(np.random.choice(valid_indices))
            
            active_id = self.env._get_active_player_id()
            mapped = self.env._map_action(action_idx, active_id)
            actions.append((active_id, mapped))
            
            self.env.step(action_idx)
            
        incremental_state = self.env.state
        incremental_history = list(self.env.history)
        
        # Now verify the same history from scratch
        full_result = sint_solver.verify_linear(["P1", "P2", "P3", "P4"], 12345, actions)
        full_state = full_result['final_state']
        
        self.assertEqual(incremental_state['sequence_id'], full_state['sequence_id'])
        self.assertEqual(incremental_state['turn_count'], full_state['turn_count'])
        self.assertEqual(incremental_state['phase'], full_state['phase'])

    def test_invalid_action_ends_episode(self):
        self.env.reset()
        
        # Action 11 is 'Bake'. P1 starts in Room 2 (Dormitory). 
        # Kitchen is Room 5. Baking in Room 2 is invalid.
        action_idx = 11
        active_id = self.env._get_active_player_id()
        
        # Verify it's actually masked out
        masks = self.env.action_masks()
        self.assertFalse(masks[action_idx], f"Action {action_idx} (Bake) should be masked for {active_id} in Room 2")
        
        # Step should now raise an AssertionError
        with self.assertRaises(AssertionError) as cm:
            self.env.step(action_idx)
        
        self.assertIn("MASKED as invalid", str(cm.exception))

    def test_observation_updates_on_move(self):
        obs, _ = self.env.reset()
        
        # P1 is at index 0 of the players section (Global: 8, Rooms: 90, Players: index 0)
        # Each player has 9 values. P1 values start at index 98.
        # Player data: [room_id, hp, ap, inv*5, is_active]
        p1_room_idx = 98 
        
        initial_room = obs[p1_room_idx]
        
        # Indices 0-9 are Move to room 0-9.
        # We are in Room 2, neighbor of Room 2 is Room 0.
        action_idx = 0 
        
        obs, reward, done, truncated, info = self.env.step(action_idx)
        
        self.assertEqual(obs[p1_room_idx], 0.0, f"P1 room_id in observation should be 0 after move. Got {obs[p1_room_idx]}")
        self.assertNotEqual(obs[p1_room_idx], initial_room)

    def test_reward_range(self):
        self.env.reset()
        
        # Take 10 random valid steps and ensure rewards aren't astronomical
        for _ in range(10):
            masks = self.env.action_masks()
            valid_indices = np.where(masks)[0]
            if len(valid_indices) == 0:
                break
                
            action_idx = int(np.random.choice(valid_indices))
            
            obs, reward, done, truncated, info = self.env.step(action_idx)
            # Dense rewards are now in a smaller range (rl_score)
            self.assertLess(abs(reward), 200.0, f"Reward {reward} seems too large")
            if done: break

    def test_active_player_index_in_obs(self):
        obs, _ = self.env.reset()
        
        # P1 is active first. Active player bit is index 8 of player data.
        # P1: 98 + 8 = 106
        self.assertEqual(obs[106], 1.0, "P1 should be marked as active in obs")
        self.assertEqual(obs[106+9], 0.0, "P2 should not be marked as active in obs")
        
        # Move P1 to Room 0 then Room 1 (Room 2 -> 0 -> 1)
        # Each Move costs 1 AP. P1 has 2 AP.
        self.env.step(0) # Move to 0
        self.env.step(1) # Move to 1
        
        obs = self.env._get_obs()
        self.assertEqual(obs[106], 0.0, "P1 should no longer be active in obs")
        self.assertEqual(obs[106+9], 1.0, "P2 should now be marked as active in obs")

    def test_situation_multi_hot(self):
        obs, _ = self.env.reset()
        
        # Active situations start at index 8 (Global) + 90 (Rooms) + 54 (Players) = 152
        # CardId has 49 variants.
        situation_slice = obs[152:152+49]
        
        active_count_in_state = len(self.env.state['active_situations'])
        active_count_in_obs = np.sum(situation_slice)
        
        self.assertEqual(active_count_in_obs, active_count_in_state, 
                         "Observation situation count doesn't match state")

    def test_bake_updates_obs(self):
        self.env.reset(seed=12345)
        # Move P1 to Kitchen (Room 2 -> 0 -> 5)
        self.env.step(0) # Move to 0
        self.env.step(5) # Move to 5
        
        # Advance to Round 2
        while self.env.state['turn_count'] == 1:
            self.env.step(45) # Pass
            
        self.assertEqual(self.env.state['turn_count'], 2)
        self.assertEqual(self.env._get_active_player_id(), 'P1')
        
        # Room 5 data starts at 8 + 5*9 = 53.
        # Item: [Peppernut, Extinguisher, Keychain, Wheelbarrow, Mitre]
        # Peppernut is at offset 2.
        kitchen_peppernut_obs_idx = 53 + 2
        
        initial_nuts = self.env._get_obs()[kitchen_peppernut_obs_idx]
        
        # Action 11 is Bake
        self.env.step(11)
        
        new_obs = self.env._get_obs()
        self.assertGreater(new_obs[kitchen_peppernut_obs_idx], initial_nuts, "Kitchen peppernuts should increase after Bake")

    def test_pickup_updates_obs(self):
        self.env.reset(seed=12345)
        # Storage is Room 9. Move P1 there (Room 2 -> 0 -> 9)
        self.env.step(0)
        self.env.step(9) # Round might advance if P1 was the only one with AP, but usually others have AP.
        
        # Advance to Round 2
        while self.env.state['turn_count'] == 1:
            self.env.step(45) # Pass
            
        self.assertEqual(self.env.state['turn_count'], 2)
        
        # Room 9 Peppernut index: 8 + 9*9 + 2 = 91
        # P1 Inventory Peppernut index: 98 + 3 = 101 (Peppernut is 1st item type, so offset 3)
        
        room_nuts_idx = 91
        player_nuts_idx = 101
        
        obs = self.env._get_obs()
        initial_room_nuts = obs[room_nuts_idx]
        initial_player_nuts = obs[player_nuts_idx]
        
        # Action 17 is PickUp Peppernut
        self.env.step(17)
        
        new_obs = self.env._get_obs()
        # Item normalization is / 10.0 for rooms, / 5.0 for players
        self.assertAlmostEqual(new_obs[room_nuts_idx], initial_room_nuts - 0.1)
        self.assertAlmostEqual(new_obs[player_nuts_idx], initial_player_nuts + 0.2)

    def test_shoot_updates_obs(self):
        self.env.reset(seed=12345)
        initial_hp = self.env.state['enemy']['hp']
        
        # Room IDs
        STORAGE = 9
        KITCHEN = 5
        CANNONS = 6
        HALLWAY = 0
        
        for i in range(200): # More steps for complex situation resolution
            state = self.env.state
            p1 = state['players']['P1']
            room_id = p1['room_id']
            ap = p1['ap']
            inventory = p1['inventory']
            active_situations = [s['id'] for s in state['active_situations']]
            
            # print(f"STEP {i}: Round={state['turn_count']}, Room={room_id}, AP={ap}, Situations={active_situations}")

            if ap == 0:
                self.env.step(45) # Pass turn
                continue
                
            masks = self.env.action_masks()

            # Priority 1: Solve SugarRush if it exists and we want to shoot
            if 'SugarRush' in active_situations:
                if room_id == KITCHEN:
                    if masks[10]: self.env.step(10)
                    else: self.env.step(45)
                elif room_id == HALLWAY:
                    if masks[KITCHEN]: self.env.step(KITCHEN)
                    else: self.env.step(45)
                else:
                    if masks[HALLWAY]: self.env.step(HALLWAY)
                    else: self.env.step(45)
                continue
            
            # Priority 2: Get Ammo
            if 'Peppernut' not in inventory:
                if room_id == STORAGE:
                    if masks[17]: self.env.step(17)
                    else: self.env.step(45)
                elif room_id == HALLWAY:
                    if masks[STORAGE]: self.env.step(STORAGE)
                    else: self.env.step(45)
                else:
                    if masks[HALLWAY]: self.env.step(HALLWAY)
                    else: self.env.step(45)
                continue
                
            # Priority 3: Move to Cannons
            if room_id != CANNONS:
                target = CANNONS if room_id == HALLWAY else HALLWAY
                if masks[target]:
                    self.env.step(target)
                else:
                    self.env.step(45)
                continue
                
            # Priority 4: Shoot!
            if masks[12]:
                obs, reward, done, truncated, info = self.env.step(12)
                if self.env.state['enemy']['hp'] < initial_hp:
                    break # Success!
                if done: break
            else:
                self.env.step(45)

        new_obs = self.env._get_obs()
        # enemy_hp normalization is / 100.0
        self.assertLess(new_obs[3], initial_hp / 100.0, f"Enemy HP should decrease. Final HP: {self.env.state['enemy']['hp']}")

    def test_extinguish_updates_obs(self):
        # We need a seed where a fire exists or we spawn one
        self.env.reset(seed=12345)
        # Room 4 (Engine) starts with Fire in this seed
        # Room 4 data index: 8 + 4*9 = 44. Fire is offset 0.
        fire_idx = 44
        
        # Move P1 to Engine (Room 2 -> 0 -> 4)
        self.env.step(0); self.env.step(4)
        
        # Round 2 starts after 2 moves. P1 is at Room 4.
        while self.env.state['turn_count'] == 1: self.env.step(45)
        
        initial_fire = self.env._get_obs()[fire_idx]
        self.assertGreater(initial_fire, 0.0, "Engine room should have fire for this test")
        
        # Action 15 is Extinguish
        self.env.step(15)
        
        new_obs = self.env._get_obs()
        self.assertLess(new_obs[fire_idx], initial_fire, "Fire count should decrease in observation after Extinguish")

    def test_ap_lifecycle_in_obs(self):
        obs, _ = self.env.reset()
        # P1 AP is at index 98 (P1 start) + 2 = 100
        ap_idx = 100
        
        # player_ap normalization is / 4.0
        self.assertEqual(obs[ap_idx], 2.0 / 4.0, "P1 should start with 2 AP (normalized)")
        
        # Use two DISTINCT moves to ensure AP is spent
        self.env.step(0) # Room 2 -> 0 (1 AP)
        obs = self.env._get_obs()
        self.assertEqual(obs[ap_idx], 1.0 / 4.0, "P1 should have 1 AP after move 1 (normalized)")
        
        self.env.step(2) # Room 0 -> 2 (1 AP)
        obs = self.env._get_obs()
        self.assertEqual(obs[ap_idx], 0.0 / 4.0, "P1 should have 0 AP after move 2 (normalized)")
        
        # Advance round.
        while self.env.state['turn_count'] == 1: self.env.step(45)
        
        obs = self.env._get_obs()
        self.assertEqual(obs[ap_idx], 2.0 / 4.0, "P1 AP should reset to 2 in Round 2 (normalized)")

    def test_hazard_masking(self):
        self.env.reset(seed=12345)
        # Move P1 to Engine (Room 4), which has Fire.
        self.env.step(0); self.env.step(4)
        while self.env.state['players']['P1']['ap'] == 0: self.env.step(45)
        
        # In a room with Fire, P1 should NOT be able to do standard actions 
        # (Assuming Bake/Shoot/etc were valid there, but here let's just check masking)
        masks = self.env.action_masks()
        # For this test, just verify that Extinguish (15) IS valid and 
        # standard actions like Shoot (12) are NOT if they were possible.
        # But even simpler: standard Room 4 has Engine system. 
        # Action 14 is EvasiveManeuvers (Engine).
        self.assertTrue(masks[15], "Extinguish should be valid in a burning room")
        # The game logic prohibits system actions if hazards are present.
        self.assertFalse(masks[14], "System action (EvasiveManeuvers) should be masked in a burning room")

    def test_reward_extinguish_fire(self):
        self.env.reset(seed=12345)
        # Engine (Room 4) has fire in this seed.
        self.env.step(0); self.env.step(4) # Move P1 to Room 4
        while self.env.state['players']['P1']['ap'] == 0: self.env.step(45) # Advance to when P1 has AP again
        
        # Action 15 is Extinguish
        obs, reward, done, truncated, info = self.env.step(15)
        
        # According to rl.rs, fire_extinguish_reward is 1.0
        # Plus a small survival reward based on hull (20/20 * 0.1 = 0.1)
        # Minus step penalty (0.01)
        # Total should be around 1.09
        self.assertGreater(reward, 0.5, f"Expected positive reward for extinguishing fire, got {reward}")
        self.assertLess(reward, 2.0)

    def test_rl_scoring_logic(self):
        # Directly test the Rust scoring function
        obs, _ = self.env.reset(seed=12345)
        parent_state = self.env.state
        
        # Create a "current" state where boss HP is lower
        import copy
        current_state = copy.deepcopy(parent_state)
        current_state['enemy']['hp'] -= 1
        
        # compute_score_rl(parent, current, history)
        reward = sint_solver.compute_score_rl(parent_state, current_state, [])
        
        # boss_damage_reward is 10.0. 
        # Survival reward is (20/20 * 0.1) = 0.1.
        # step_penalty is 0.01.
        # total = 10.0 + 0.1 - 0.01 = 10.09
        self.assertAlmostEqual(reward, 10.09, places=2)

    def test_reward_repair_system(self):
        self.env.reset(seed=12345)
        parent_state = self.env.state
        
        import copy
        current_state = copy.deepcopy(parent_state)
        
        # Room keys are integers in the dictionary from verify_linear
        rooms_p = parent_state['map']['rooms']
        rooms_c = current_state['map']['rooms']
        
        room_id = 4 # Engine
        rooms_p[room_id]['is_broken'] = True
        reward = sint_solver.compute_score_rl(parent_state, current_state, [])
        # system_repair_reward is 1.0. 
        # Survival is 0.1. Step is 0.01.
        # total = 1.0 + 0.1 - 0.01 = 1.09
        self.assertAlmostEqual(reward, 1.09, places=2)

    def test_terminal_state_victory_reward(self):
        obs, _ = self.env.reset(seed=12345)
        parent_state = self.env.state
        
        import copy
        current_state = copy.deepcopy(parent_state)
        current_state['phase'] = 'Victory'
        
        reward = sint_solver.compute_score_rl(parent_state, current_state, [])
        # victory_reward is 100.0. No survival/step penalty in terminal state branch.
        self.assertEqual(reward, 100.0)

    def test_terminal_state_gameover_reward(self):
        obs, _ = self.env.reset(seed=12345)
        parent_state = self.env.state
        
        import copy
        current_state = copy.deepcopy(parent_state)
        current_state['phase'] = 'GameOver'
        
        reward = sint_solver.compute_score_rl(parent_state, current_state, [])
        # defeat_penalty is -100.0.
        self.assertEqual(reward, -100.0)


    def test_action_masks_no_ap(self):
        self.env.reset()
        # Drain P1's AP
        self.env.step(0)
        self.env.step(2)
        
        # Now P2 should be active.
        active_id = self.env._get_active_player_id()
        self.assertEqual(active_id, 'P2')
        
        masks = self.env.action_masks()
        self.assertTrue(masks.any())
        
    def test_invalid_seed_initialization(self):
        # Ensure different seeds produce different initial states
        env1 = SintEnv(seed=1)
        env2 = SintEnv(seed=2)
        obs1, _ = env1.reset()
        obs2, _ = env2.reset()
        # Boss HP or situations might differ
        self.assertFalse(np.array_equal(obs1, obs2))

    def test_observation_scaling_limits(self):
        obs, _ = self.env.reset()
        # All normalized values should be roughly within [-1.5, 1.5]
        self.assertTrue(np.all(obs >= -1.1), f"Obs contains values below -1.1: {obs[obs < -1.1]}")
        self.assertTrue(np.all(obs <= 1.1), f"Obs contains values above 1.1: {obs[obs > 1.1]}")

    def test_juggling_penalty(self):
        self.env.reset(seed=12345)
        parent_state = self.env.state
        
        import copy
        current_state = copy.deepcopy(parent_state)
        
        # Player P1 starts with 0 items. 
        # Add an item to parent, then remove it in current (representing a Drop).
        # We need to make sure the scoring function sees the decrease.
        
        parent_state['players']['P1']['inventory'] = ['Peppernut']
        current_state['players']['P1']['inventory'] = []
        
        # Compute reward
        reward = sint_solver.compute_score_rl(parent_state, current_state, [])
        
        # item_drop_penalty is 0.15. 
        # Survival is 0.1. Step is 0.01.
        # total = -0.15 + 0.1 - 0.01 = -0.06
        self.assertAlmostEqual(reward, -0.06, places=2)

if __name__ == '__main__':
    unittest.main()
