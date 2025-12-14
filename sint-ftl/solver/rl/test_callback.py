import unittest
import numpy as np
import os
import sys
from sb3_contrib import MaskablePPO
from stable_baselines3.common.env_util import make_vec_env

# Add current directory to path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from env import SintEnv
from train import TUICallback

class TestCallbackRewardCapture(unittest.TestCase):
    def test_reward_and_action_capture_logic(self):
        """
        Verify that TUICallback correctly captures the mean reward and action distribution.
        """
        eval_env = SintEnv(num_players=4)
        callback = TUICallback(eval_env, eval_freq=1000)
        
        # Mock rewards and actions for 4 parallel environments
        mock_rewards = np.array([1.5, 2.5, 0.5, 3.5])
        mock_actions = np.array([12, 12, 0, 45]) # Shoot, Shoot, Move, Pass
        
        callback.locals = {
            'rewards': mock_rewards,
            'actions': mock_actions
        }
        callback.n_calls = 1
        callback.is_tty = False
        callback._stop_training = False
        
        # Mock model and env for the _on_step logic
        class MockEnv:
            def get_attr(self, name): return None
        callback.model = type('obj', (object,), {'get_env': lambda: MockEnv()})
        
        callback._on_step()
        
        # Verify Reward
        expected_mean = float(np.mean(mock_rewards))
        self.assertEqual(callback.stats["latest_reward"], expected_mean)
        
        # Verify Action Distribution
        counts = callback.stats["action_counts"]
        self.assertEqual(counts.get("Shoot"), 2)
        self.assertEqual(counts.get("Move"), 1)
        self.assertEqual(counts.get("Pass"), 1)
        
        print(f"Captured actions correctly: {counts}")

    def test_reward_breakdown_capture(self):
        """Verify that TUICallback correctly captures the detailed reward breakdown."""
        eval_env = SintEnv(num_players=4)
        callback = TUICallback(eval_env, eval_freq=1000)
        callback._stop_training = False
        
        # Mock breakdown from a sub-environment
        mock_breakdown = {
            "vitals": 0.1,
            "hazards": 1.0,
            "offense": 2.0,
            "total": 3.1
        }
        
        # Mock training_env requirements
        class MockEnv:
            def get_attr(self, name):
                if name == "last_details": return [mock_breakdown]
                if name == "history": return [[("P1", "Move 0")]]
                if name == "state": return [{"hull_integrity": 20, "enemy": {"hp": 5}, "phase": "TacticalPlanning"}]
                return None
        
        callback.model = type('obj', (object,), {'get_env': lambda: MockEnv()})
        callback.locals = {
            'rewards': [0.1],
            'dones': [False],
            'actions': [0]
        }
        callback.n_calls = 1
        
        callback._on_step()
        
        self.assertEqual(callback.stats["latest_reward_breakdown"], mock_breakdown)
        print(f"Captured breakdown correctly: {callback.stats['latest_reward_breakdown']}")

    def test_vectorized_action_handling(self):
        """Verify that record_action handles various input types."""
        eval_env = SintEnv(num_players=4)
        callback = TUICallback(eval_env, eval_freq=1000)
        
        # Test scalar
        callback._record_action(12)
        self.assertEqual(callback.stats["action_counts"]["Shoot"], 1)
        
        # Test numpy array
        callback._record_action(np.array([0, 11]))
        self.assertEqual(callback.stats["action_counts"]["Move"], 1)
        self.assertEqual(callback.stats["action_counts"]["Bake"], 1)
        
        # Test list
        callback._record_action([15, 15, 16])
        self.assertEqual(callback.stats["action_counts"]["Extinguish"], 2)
        self.assertEqual(callback.stats["action_counts"]["Repair"], 1)

    def test_evaluation_history_accumulation(self):
        """Verify that evaluation results are correctly stored in history."""
        eval_env = SintEnv(num_players=4, seed=12345, test_mode=True)
        callback = TUICallback(eval_env, eval_freq=10)
        callback.is_tty = False
        callback.n_calls = 10
        
        # Mock model.predict to always return Pass (45)
        callback.model = type('obj', (object,), {
            'predict': lambda obs, action_masks, deterministic: (45, None),
            'save': lambda path: None
        })
        
        callback._run_evaluation()
        
        self.assertEqual(len(callback.eval_history), 1)
        step, reward, win_rate, fs = callback.eval_history[0]
        self.assertEqual(step, 10)
        self.assertIsInstance(reward, float)
        self.assertIsInstance(win_rate, float)
        self.assertIn("H=", fs)

if __name__ == '__main__':
    unittest.main()
