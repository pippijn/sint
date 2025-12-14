import argparse
import os
import time
import sys
import select
import tty
import termios
import signal
import numpy as np
import threading
import traceback
from datetime import datetime
from sb3_contrib import MaskablePPO
from stable_baselines3.common.env_util import make_vec_env
from stable_baselines3.common.callbacks import CheckpointCallback, BaseCallback
from env import SintEnv
from dashboard import SintDashboard, get_total_boss_hp

class TUICallback(BaseCallback):
    def __init__(self, eval_env, eval_freq=5000, verbose=1):
        super().__init__(verbose)
        self.eval_env = eval_env
        self.eval_freq = eval_freq
        self.best_mean_reward = -float("inf")
        self.start_time = time.time()
        
        self.is_tty = sys.stdout.isatty()
        self.dashboard = SintDashboard(self.start_time) if self.is_tty else None
        
        self.lock = threading.Lock()
        self.log_file = "solver/rl/crash_report.log"
        self._stop_training = False
        
        # Clear old log
        with open(self.log_file, "w") as f:
            f.write(f"Training started at {datetime.now()}\n")
            
        self.eval_history = []
        
        self.stats = {
            "total_steps": 0,
            "best_mean_reward": -float("inf"),
            "latest_round": 0,
            "latest_trajectory": "N/A",
            "latest_trajectory_steps": 0,
            "latest_state_summary": "N/A",
            "latest_reward": 0.0,
            "current_reward": 0.0,
            "latest_reward_breakdown": {},
            "latest_seed": 0,
            "action_counts": {},
        }

        # Best Episode (Hall of Fame)
        self.best_ep_reward = -float('inf')
        self.best_ep_trajectory = "N/A"
        self.best_ep_breakdown = {}
        self.best_ep_summary = "N/A"
        self.best_ep_steps = 0
        self.best_ep_rounds = 0
        self.best_ep_seed = 0
        
        # Accumulators for parallel environments (8 envs)
        self.current_ep_rewards = [0.0] * 8
        self.current_ep_breakdowns = [{} for _ in range(8)]
        self.current_ep_trajectories = [[] for _ in range(8)]
        self.current_ep_seeds = [0] * 8

    def _log_error(self, context, error):
        with open(self.log_file, "a") as f:
            f.write(f"[{datetime.now()}] ERROR in {context}: {error}\n")
            f.write(traceback.format_exc())
            f.write("\n" + "="*40 + "\n")

    def _update_live(self):
        try:
            if self.is_tty and self.dashboard:
                best_ep = {
                    "reward": self.best_ep_reward,
                    "breakdown": self.best_ep_breakdown,
                    "trajectory": self.best_ep_trajectory,
                    "summary": self.best_ep_summary,
                    "steps": self.best_ep_steps,
                    "rounds": self.best_ep_rounds,
                    "seed": self.best_ep_seed
                }
                with self.lock:
                    self.stats["total_steps"] = self.n_calls
                    self.stats["best_mean_reward"] = self.best_mean_reward
                    self.stats["current_reward"] = self.current_ep_rewards[0]
                    self.stats["latest_seed"] = self.current_ep_seeds[0]
                    self.dashboard.update(self.stats, best_ep, self.eval_history)
            elif not self.is_tty and self.n_calls % 1000 == 0:
                elapsed = time.time() - self.start_time
                fps = self.n_calls / elapsed if elapsed > 0 else 0
                best = f"{self.best_mean_reward:.2f}" if self.best_mean_reward != -float("inf") else "N/A"
                
                with self.lock:
                    latest_r = self.stats.get("latest_reward", 0.0)
                    breakdown = self.stats.get("latest_reward_breakdown", {})
                    top_driver = "N/A"
                    if breakdown:
                        drivers = [(k, v) for k, v in breakdown.items() if k != "total" and abs(v) > 0.0001]
                        if drivers:
                            best_k, best_v = max(drivers, key=lambda x: abs(x[1]))
                            top_driver = f"{best_k}:{best_v:+.2f}"

                    total_actions = sum(self.stats["action_counts"].values())
                    act_str = "N/A"
                    if total_actions > 0:
                        sorted_actions = sorted(self.stats["action_counts"].items(), key=lambda x: x[1], reverse=True)
                        act_str = " ".join([f"{a}:{c/total_actions*100:.0f}%" for a, c in sorted_actions[:8]])

                    print(
                        f"[{datetime.now().strftime('%H:%M:%S')}] "
                        f"Step: {self.n_calls:8,} | "
                        f"FPS: {fps:6.1f} | "
                        f"Rew: {latest_r:+.4f} ({top_driver}) | "
                        f"Best Ep: {self.best_ep_reward:8.1f} | "
                        f"Best Mean: {best:>8} | "
                        f"Acts: {act_str}"
                    )
                    self.stats["action_counts"] = {}
        except Exception as e:
            self._log_error("_update_live", e)

    def _check_quit(self):
        if not self.is_tty: return False
        while select.select([sys.stdin], [], [], 0)[0]:
            char = sys.stdin.read(1)
            if char.lower() == 'q': return True
        return False

    def _run_evaluation(self):
        try:
            rewards = []
            final_states = []
            wins = 0
            for _ in range(5):
                obs, _ = self.eval_env.reset()
                terminated = False
                truncated = False
                total_reward = 0
                while not (terminated or truncated):
                    action_masks = self.eval_env.action_masks()
                    action, _ = self.model.predict(obs, action_masks=action_masks, deterministic=True)
                    obs, reward, terminated, truncated, _ = self.eval_env.step(action)
                    total_reward += reward
                
                fs = self.eval_env.state
                if fs['phase'] == 'Victory': wins += 1
                
                rewards.append(total_reward)
                total_boss_hp = get_total_boss_hp(fs)
                final_states.append(f"H={fs['hull_integrity']}, B={total_boss_hp}, R={fs['turn_count']}")
            
            mean_reward = sum(rewards) / len(rewards)
            win_rate = (wins / 5) * 100
            with self.lock:
                self.eval_history.append((self.n_calls, mean_reward, win_rate, final_states[0]))
            
            if not self.is_tty:
                print(f"📊 [Eval at {self.n_calls:,}] Mean Reward: {mean_reward:+.2f} | Win Rate: {win_rate:3.0f}% | Sample: {final_states[0]}")
            
            if mean_reward > self.best_mean_reward:
                if not self.is_tty: print(f"✨ New Best Model! ({mean_reward:.2f})")
                self.best_mean_reward = mean_reward
                self.model.save("solver/rl/models/ppo_sint_best")
        except Exception as e:
            self._log_error("_run_evaluation", e)

    def _on_training_start(self) -> None:
        if self.is_tty and self.dashboard:
            self.old_settings = termios.tcgetattr(sys.stdin)
            tty.setcbreak(sys.stdin.fileno())
            def signal_handler(sig, frame):
                self._restore_terminal()
                print("\nInterrupt received. Saving model...")
                self._stop_training = True
            signal.signal(signal.SIGINT, signal_handler)
            signal.signal(signal.SIGTERM, signal_handler)
            self.dashboard.start()
        else:
            print("🚀 Starting training (Non-TTY mode)")
            def signal_handler(sig, frame):
                print("\nInterrupt received. Stopping...")
                self._stop_training = True
            signal.signal(signal.SIGINT, signal_handler)
            signal.signal(signal.SIGTERM, signal_handler)
        
        self._stop_training = False
        self._update_live()
        self._run_evaluation()
        self._update_live()

    def _restore_terminal(self):
        if self.dashboard: self.dashboard.stop()
        if hasattr(self, 'old_settings') and self.old_settings:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, self.old_settings)
            self.old_settings = None

    def _on_training_end(self) -> None:
        self._restore_terminal()
        print("✅ Training finished")

    def _on_step(self) -> bool:
        try:
            if self._stop_training or self._check_quit(): return False
            rewards = self.locals.get('rewards', [])
            dones = self.locals.get('dones', [])
            actions = self.locals.get('actions', [])
            infos = self.locals.get('infos', [])
            
            with self.lock:
                if len(rewards) > 0: self.stats["latest_reward"] = float(np.mean(rewards))
                if len(actions) > 0: self._record_action(actions)

            if self.training_env is not None:
                try:
                    details = self.training_env.get_attr("last_details")
                    histories = self.training_env.get_attr("history")
                    states = self.training_env.get_attr("state")
                    seeds = self.training_env.get_attr("initial_seed")

                    with self.lock:
                        if seeds:
                            self.current_ep_seeds = seeds

                        if details and len(details) == len(rewards):
                            for i in range(len(details)):
                                is_terminal = dones[i] if i < len(dones) else False
                                info = infos[i] if i < len(infos) else {}
                                step_det = info.get("terminal_details", details[i])
                                step_rew = rewards[i] if i < len(rewards) else 0.0
                                
                                while len(self.current_ep_rewards) <= i:
                                    self.current_ep_rewards.append(0.0)
                                    self.current_ep_breakdowns.append({})
                                    self.current_ep_trajectories.append([])

                                self.current_ep_rewards[i] += step_rew
                                for k, v in step_det.items():
                                    self.current_ep_breakdowns[i][k] = self.current_ep_breakdowns[i].get(k, 0.0) + v
                                
                                if is_terminal:
                                    if self.current_ep_rewards[i] > self.best_ep_reward:
                                        self.best_ep_reward = self.current_ep_rewards[i]
                                        self.best_ep_breakdown = self.current_ep_breakdowns[i].copy()
                                        self.best_ep_seed = self.current_ep_seeds[i]
                                        term_history = info.get("terminal_history")
                                        if term_history:
                                            self.best_ep_steps = len(term_history)
                                            log_lines = [f"  {p}: {a}" for p, a in term_history[-20:]]
                                            self.best_ep_trajectory = "\n".join(log_lines)
                                        s_final = info.get("terminal_state")
                                        if s_final:
                                            self.best_ep_rounds = s_final.get('turn_count', 0)
                                            total_boss_hp = get_total_boss_hp(s_final)
                                            self.best_ep_summary = (f"Hull: {s_final['hull_integrity']} | Total Boss HP: {total_boss_hp} | Phase: {s_final['phase']}")
                                    
                                    self.current_ep_rewards[i] = 0.0
                                    self.current_ep_breakdowns[i] = {}
                                    self.current_ep_trajectories[i] = []

                            self.stats["latest_reward_breakdown"] = self.current_ep_breakdowns[0].copy()

                        if states and states[0]:
                            s = states[0]
                            self.stats["latest_round"] = s.get("turn_count", 0)
                            self.stats["latest_state_summary"] = (f"Hull: {s['hull_integrity']} | Total Boss HP: {get_total_boss_hp(s)} | Phase: {s['phase']} | Active Players: {sum(1 for p in s['players'].values() if p['ap'] > 0)}")

                        if histories and histories[0]:
                            flat_history = histories[0]
                            self.stats["latest_trajectory"] = "\n".join([f"  {p}: {a}" for p, a in flat_history[-20:]])
                            self.stats["latest_trajectory_steps"] = len(flat_history)
                            
                except Exception as e:
                    self._log_error("_on_step_env", e)

            if self.n_calls % 100 == 0: self._update_live()
            if self.n_calls % self.eval_freq == 0:
                self._run_evaluation()
                self._update_live()
            return True
        except Exception as e:
            self._log_error("_on_step", e)
            return False

    def _record_action(self, act):
        if isinstance(act, (list, np.ndarray)):
            for a in act: self._record_action(a)
            return
        act_name = "Other"
        if act < 10: act_name = "Move"
        elif act == 10: act_name = "Interact"
        elif act == 11: act_name = "Bake"
        elif act == 12: act_name = "Shoot"
        elif act == 13: act_name = "Shields"
        elif act == 14: act_name = "Evasive"
        elif act == 15: act_name = "Extinguish"
        elif act == 16: act_name = "Repair"
        elif 17 <= act <= 21: act_name = "PickUp"
        elif 22 <= act <= 26: act_name = "Drop"
        elif 27 <= act <= 32: act_name = "Throw"
        elif 33 <= act <= 38: act_name = "FirstAid"
        elif 39 <= act <= 44: act_name = "Revive"
        elif act == 45: act_name = "Pass"
        self.stats["action_counts"][act_name] = self.stats["action_counts"].get(act_name, 0) + 1

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--steps", type=int, default=100000)
    parser.add_argument("--output", type=str, default="solver/rl/models/ppo_sint")
    parser.add_argument("--num-players", type=int, default=4)
    args = parser.parse_args()

    def make_env():
        return SintEnv(num_players=args.num_players)

    env = make_vec_env(make_env, n_envs=8)

    model_path = args.output
    if not model_path.endswith(".zip"): model_path += ".zip"

    if os.path.exists(model_path):
        print(f"📦 Loading existing model from {model_path}")
        model = MaskablePPO.load(model_path, env=env, tensorboard_log="solver/rl/logs/")
    else:
        print("🆕 Creating new model")
        model = MaskablePPO("MlpPolicy", env, verbose=1, tensorboard_log="solver/rl/logs/", learning_rate=3e-4, n_steps=4096, batch_size=256, n_epochs=10, gamma=0.99, gae_lambda=0.95, clip_range=0.2, ent_coef=0.01)

    eval_env = SintEnv(num_players=args.num_players)
    tui_callback = TUICallback(eval_env, eval_freq=5000)
    checkpoint_callback = CheckpointCallback(save_freq=20000, save_path="solver/rl/models/", name_prefix="ppo_sint_checkpoint")

    try:
        model.learn(total_timesteps=args.steps, callback=[checkpoint_callback, tui_callback])
    except KeyboardInterrupt:
        print("\nInterrupted by user")
    finally:
        model.save(args.output)
        print(f"✅ Model saved to {args.output}")

if __name__ == "__main__":
    main()
