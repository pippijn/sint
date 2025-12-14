import argparse
import os
from sb3_contrib import MaskablePPO
from stable_baselines3.common.env_util import make_vec_env
from stable_baselines3.common.callbacks import CheckpointCallback, BaseCallback
from env import SintEnv

class BestModelCallback(BaseCallback):
    def __init__(self, eval_env, eval_freq=10000, verbose=1):
        super().__init__(verbose)
        self.eval_env = eval_env
        self.eval_freq = eval_freq
        self.best_mean_reward = -float("inf")

    def _on_step(self) -> bool:
        if self.n_calls % self.eval_freq == 0:
            # Run 3 evaluation episodes
            rewards = []
            final_stats = []
            for _ in range(3):
                obs, _ = self.eval_env.reset()
                done = False
                total_reward = 0
                while not done:
                    action_masks = self.eval_env.action_masks()
                    action, _ = self.model.predict(obs, action_masks=action_masks, deterministic=True)
                    obs, reward, done, _, _ = self.eval_env.step(action)
                    total_reward += reward
                rewards.append(total_reward)
                fs = self.eval_env.state
                final_stats.append(f"Hull={fs.hull_integrity}, BossHP={fs.enemy.hp}, Phase={fs.phase.value}")
            
            mean_reward = sum(rewards) / len(rewards)
            if mean_reward > self.best_mean_reward:
                print(f"\n✨ NEW BEST! Mean Reward: {mean_reward:.2f} (was {self.best_mean_reward:.2f})")
                print(f"   Sample Stats: {final_stats[0]}")
                self.best_mean_reward = mean_reward
                # Save best so far
                self.model.save("solver/rl/models/ppo_sint_best")
        return True

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--steps", type=int, default=100000)
    parser.add_argument("--output", type=str, default="solver/rl/models/ppo_sint")
    parser.add_argument("--num-players", type=int, default=4)
    args = parser.parse_args()

    # Create environment
    def make_env():
        return SintEnv(num_players=args.num_players)

    env = make_vec_env(make_env, n_envs=4)

    # Initialize model using MaskablePPO
    model = MaskablePPO(
        "MlpPolicy",
        env,
        verbose=1,
        tensorboard_log="solver/rl/logs/",
        learning_rate=3e-4,
        n_steps=2048,
        batch_size=64,
        n_epochs=10,
        gamma=0.99,
        gae_lambda=0.95,
        clip_range=0.2,
        ent_coef=0.01,
    )

    # Setup checkpointing
    checkpoint_callback = CheckpointCallback(
        save_freq=20000,
        save_path="solver/rl/models/",
        name_prefix="ppo_sint_checkpoint"
    )
    
    # Setup best model callback
    eval_env = SintEnv(num_players=args.num_players)
    best_callback = BestModelCallback(eval_env, eval_freq=5000)

    print(f"🚀 Training for {args.steps} steps...")
    model.learn(total_timesteps=args.steps, callback=[checkpoint_callback, best_callback])

    # Save final model
    model.save(args.output)
    print(f"✅ Model saved to {args.output}")

    # --- Final Evaluation ---
    print("\n📊 Final Evaluation (5 Episodes):")
    eval_env = SintEnv(num_players=args.num_players)
    for i in range(5):
        obs, _ = eval_env.reset()
        done = False
        total_reward = 0
        steps = 0
        while not done:
            # Use action masks for deterministic prediction
            action_masks = eval_env.action_masks()
            action, _ = model.predict(obs, action_masks=action_masks, deterministic=True)
            obs, reward, done, _, _ = eval_env.step(action)
            total_reward += reward
            steps += 1
            if steps > 1000: break # Safety break
            
        final_state = eval_env.state
        print(f"Episode {i+1}: Result={final_state.phase.value}, Hull={final_state.hull_integrity}, BossHP={final_state.enemy.hp}, Steps={steps}, Reward={total_reward:.1f}")

if __name__ == "__main__":
    main()
