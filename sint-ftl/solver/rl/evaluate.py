import argparse
from sb3_contrib import MaskablePPO
from env import SintEnv
import numpy as np

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", type=str, required=True)
    parser.add_argument("--episodes", type=int, default=5)
    parser.add_argument("--num-players", type=int, default=4)
    args = parser.parse_args()

    env = SintEnv(num_players=args.num_players)
    model = MaskablePPO.load(args.model)

    for i in range(args.episodes):
        obs, _ = env.reset()
        done = False
        total_reward = 0
        steps = 0
        
        print(f"\n--- Episode {i+1} ---")
        while not done:
            action_masks = env.action_masks()
            action, _states = model.predict(obs, action_masks=action_masks, deterministic=True)
            obs, reward, done, truncated, info = env.step(action)
            total_reward += reward
            steps += 1
            
            if steps % 10 == 0:
                print(f"Step {steps}: Hull={env.state.hull_integrity}, Boss HP={env.state.enemy.hp}, Phase={env.state.phase}")
        
        print(f"Finished: Reward={total_reward}, Steps={steps}, Final Phase={env.state.phase}")

if __name__ == "__main__":
    main()

