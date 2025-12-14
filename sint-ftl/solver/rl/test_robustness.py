import unittest
import numpy as np
import os
import sys
import uuid
from typing import Dict, Any, List, Tuple

# Add current directory to path to import env
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from env import SintEnv

class TestSintEnvRobustness(unittest.TestCase):
    def setUp(self):
        self.env = SintEnv(num_players=4, seed=12345)

    def test_history_initialization(self):
        """Verify history is initialized as a flat empty list."""
        self.env.reset()
        self.assertIsInstance(self.env.history, list)
        self.assertEqual(len(self.env.history), 0)
        
        # Take a step and verify history appends correctly
        active_id = self.env._get_active_player_id()
        self.env.step(45) # Pass
        self.assertEqual(len(self.env.history), 1)
        self.assertEqual(self.env.history[0][0], active_id)

    def test_invalid_action_mask_consistency(self):
        """
        Verify that our action_masks and the backend remain in sync.
        The step function now assumes action_masks is the source of truth.
        """
        self.env.reset()
        masks = self.env.action_masks()
        
        # Pick an action that is MASKED OUT (e.g., Shoot in a room with no ammo/cannons)
        # P1 starts in Room 2. Shoot is 12.
        self.assertFalse(masks[12], "Shoot should be masked for P1 in Room 2 at start")
        
        # Verify that choosing it directly raises an AssertionError (as intended by our refactor)
        with self.assertRaises(AssertionError):
            self.env.step(12)

    def test_observation_space_boundaries_normalized(self):
        """Verify that normalized observations stay within a reasonable range."""
        obs, _ = self.env.reset()
        
        # Check specific normalized values
        # Hull: 20/20 = 1.0
        self.assertAlmostEqual(obs[0], 1.0)
        # Turn count: 1/100 = 0.01
        self.assertAlmostEqual(obs[1], 0.01)
        
        # Take some random steps and check range
        for _ in range(50):
            masks = self.env.action_masks()
            valid_indices = np.where(masks)[0]
            if len(valid_indices) == 0: break
            
            action_idx = int(np.random.choice(valid_indices))
            obs, _, done, _, _ = self.env.step(action_idx)
            
            # All normalized values should be in a healthy range for NN
            self.assertTrue(np.all(obs >= -1.5) and np.all(obs <= 2.0), 
                            f"Observation out of range: {obs[obs < -1.5]} or {obs[obs > 2.0]}")
            if done: break

    def test_seed_randomization_non_test_mode(self):
        """Verify that reset() generates new seeds when not in test_mode."""
        # Create an env in training mode (test_mode=False)
        train_env = SintEnv(num_players=4, seed=123, test_mode=False)
        
        seeds = []
        for _ in range(5):
            train_env.reset()
            seeds.append(train_env.initial_seed)
        
        # All seeds should be unique (randomized)
        self.assertEqual(len(set(seeds)), 5, f"Seeds were not randomized: {seeds}")
        self.assertNotEqual(seeds[0], 123)

    def test_seed_determinism_in_test_mode(self):
        """Verify that reset() uses the initial seed when in test_mode."""
        env = SintEnv(num_players=4, seed=54321, test_mode=True)
        env.reset()
        seed1 = env.initial_seed
        env.reset()
        seed2 = env.initial_seed
        
        self.assertEqual(seed1, 54321)
        self.assertEqual(seed2, 54321)

if __name__ == '__main__':
    unittest.main()
