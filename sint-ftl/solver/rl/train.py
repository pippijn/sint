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
from datetime import datetime
from sb3_contrib import MaskablePPO
from stable_baselines3.common.env_util import make_vec_env
from stable_baselines3.common.callbacks import CheckpointCallback, BaseCallback
from env import SintEnv

from rich.console import Console
from rich.layout import Layout
from rich.panel import Panel
from rich.live import Live
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn
from rich import box

def get_total_boss_hp(s):
    # Boss HPs from core/src/logic.rs
    boss_hps = [5, 10, 15, 20]
    level = s.get('boss_level', 0)
    if s.get('phase') == 'Victory':
        return 0
    
    # Current boss HP + all future bosses
    total = s['enemy']['hp']
    for i in range(level + 1, len(boss_hps)):
        total += boss_hps[i]
    return total

class TUICallback(BaseCallback):
    def __init__(self, eval_env, eval_freq=5000, verbose=1):
        super().__init__(verbose)
        self.eval_env = eval_env
        self.eval_freq = eval_freq
        self.best_mean_reward = -float("inf")
        self.start_time = time.time()
        self.console = Console()
        self.is_tty = self.console.is_terminal
        self.layout = self._setup_layout() if self.is_tty else None
        self.live = None
        self.old_settings = None
        self.lock = threading.Lock()
        
        self.reward_history = []
        self.eval_history = []
        
        self.stats = {
            "total_steps": 0,
            "episodes": 0,
            "mean_reward": 0.0,
            "best_reward": -float("inf"),
            "last_eval_stats": "N/A",
            "fps": 0,
            "latest_round": 0,
            "latest_trajectory": "N/A",
            "latest_state_summary": "N/A",
            "latest_reward": 0.0,
            "action_counts": {},
        }

    def _setup_layout(self):
        if not self.is_tty:
            return None
        layout = Layout()
        layout.split_column(
            Layout(name="header", size=3),
            Layout(name="main", ratio=1),
            Layout(name="footer", size=3),
        )
        layout["main"].split_row(
            Layout(name="left", ratio=1),
            Layout(name="right", ratio=2),
        )
        layout["left"].split_column(
            Layout(name="stats", size=12),
            Layout(name="actions", ratio=1),
        )
        layout["right"].split_column(
            Layout(name="eval", ratio=1),
            Layout(name="io", ratio=2),
        )
        return layout

    def _get_header(self):
        grid = Table.grid(expand=True)
        grid.add_column(justify="left", ratio=1)
        grid.add_column(justify="right", ratio=1)
        grid.add_row(
            "[bold white]🚢 Sint-FTL RL Training[/bold white]",
            f"[bold blue]{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}[/bold blue]"
        )
        return Panel(grid, style="white on blue")

    def _get_stats_panel(self):
        table = Table(box=box.SIMPLE, expand=True)
        table.add_column("Metric", style="cyan")
        table.add_column("Value", style="magenta")
        
        elapsed = time.time() - self.start_time
        fps = self.n_calls / elapsed if elapsed > 0 else 0
        
        with self.lock:
            table.add_row("Total Steps", f"{self.n_calls:,}")
            table.add_row("Elapsed Time", f"{elapsed:.1f}s")
            table.add_row("Steps/sec (FPS)", f"{fps:.1f}")
            table.add_row("Current Round", f"{self.stats['latest_round']}")
            table.add_row("Latest Reward", f"{self.stats['latest_reward']:.4f}")
            best_reward_str = f"{self.best_mean_reward:.2f}" if self.best_mean_reward != -float("inf") else "N/A"
            table.add_row("Best Mean Reward", best_reward_str)

            # Add reward trend (last 10 eval points)
            if self.eval_history:
                history_str = " ".join([f"{r:+.1f}" for _, r, _, _ in self.eval_history[-8:]])
                table.add_row("Eval Trend", history_str)

        return Panel(table, title="[bold]Training Stats[/bold]", border_style="green")

    def _get_actions_panel(self):
        table = Table(box=box.SIMPLE, expand=True)
        table.add_column("Action", style="cyan", width=12)
        table.add_column("Percent", style="magenta", width=8)
        table.add_column("Graph", style="white")

        all_action_names = [
            "Move", "Shoot", "PickUp", "Drop", "Shields", "Evasive", 
            "Extinguish", "Repair", "Bake", "Interact", "FirstAid", 
            "Revive", "Throw", "Pass", "Other"
        ]

        with self.lock:
            total_actions = sum(self.stats["action_counts"].values())
            
            # Collect data for sorting
            action_data = []
            for act_name in all_action_names:
                count = self.stats["action_counts"].get(act_name, 0)
                pct = (count / total_actions * 100) if total_actions > 0 else 0
                action_data.append((act_name, pct))
            
            # Sort by percentage descending
            action_data.sort(key=lambda x: x[1], reverse=True)

            for act_name, pct in action_data:
                # ASCII Art Graph [#####-----]
                bar_len = 10
                filled = int(pct / (100 / bar_len))
                bar = "█" * filled + "░" * (bar_len - filled)
                
                color = "green" if pct > 0 else "dim white"
                table.add_row(
                    act_name, 
                    f"{pct:5.1f}%", 
                    f"[{color}]{bar}[/]"
                )
        
        return Panel(table, title="[bold]Action Distribution[/bold]", border_style="cyan")

    def _get_eval_panel(self):
        table = Table(box=box.SIMPLE, expand=True)
        table.add_column("Step", style="cyan", width=10)
        table.add_column("Mean Rew", style="magenta")
        table.add_column("Win %", style="green")
        table.add_column("Final State (Sample)", style="white")
        
        with self.lock:
            for step, reward, win_rate, fs in self.eval_history[-5:]:
                table.add_row(f"{step:,}", f"{reward:+.2f}", f"{win_rate:3.0f}%", fs)
            
        return Panel(table, title="[bold]Evaluation History[/bold]", border_style="yellow")

    def _get_io_panel(self):
        grid = Table.grid(expand=True)
        grid.add_column(style="cyan")
        with self.lock:
            total_steps = self.stats.get("total_trajectory_steps", 0)
            if total_steps > 12:
                header = f"[bold]Latest Trajectory (Last 12 of {total_steps} steps):[/bold]"
            else:
                header = f"[bold]Latest Trajectory ({total_steps} steps):[/bold]"
            grid.add_row(header)
            grid.add_row(self.stats["latest_trajectory"])
            grid.add_row("")
            grid.add_row("[bold]Latest State Summary:[/bold]")
            grid.add_row(self.stats["latest_state_summary"])
        return Panel(grid, title="[bold]Last Input/Output[/bold]", border_style="magenta")

    def _update_live(self):
        if self.is_tty:
            # layout updates should be thread-safe too if possible, 
            # but usually the race is in the data being read.
            self.layout["header"].update(self._get_header())
            self.layout["left"]["stats"].update(self._get_stats_panel())
            self.layout["left"]["actions"].update(self._get_actions_panel())
            self.layout["right"]["eval"].update(self._get_eval_panel())
            self.layout["right"]["io"].update(self._get_io_panel())
            self.layout["footer"].update(Panel(f"Training in progress... Step {self.n_calls:,} | Press 'q' to quit", border_style="white"))
        elif not self.is_tty and self.n_calls % 1000 == 0:
            elapsed = time.time() - self.start_time
            fps = self.n_calls / elapsed if elapsed > 0 else 0
            best = f"{self.best_mean_reward:.2f}" if self.best_mean_reward != -float("inf") else "N/A"
            
            with self.lock:
                latest_r = self.stats.get("latest_reward", 0.0)
                total_actions = sum(self.stats["action_counts"].values())
                act_str = "N/A"
                if total_actions > 0:
                                    sorted_actions = sorted(self.stats["action_counts"].items(), key=lambda x: x[1], reverse=True)
                                    act_str = " ".join([f"{a}:{c/total_actions*100:.0f}%" for a, c in sorted_actions[:8]])
                                    self.console.print(
                    f"[{datetime.now().strftime('%H:%M:%S')}] "
                    f"Step: {self.n_calls:8,} | "
                    f"FPS: {fps:6.1f} | "
                    f"Latest Reward: {latest_r:+.4f} | "
                    f"Best Mean: {best:>8} | "
                    f"Acts: {act_str}"
                )
                # Reset action counts for next window to see moving distribution
                self.stats["action_counts"] = {}

    def _check_quit(self):
        if not self.is_tty:
            return False
        
        while select.select([sys.stdin], [], [], 0)[0]:
            char = sys.stdin.read(1)
            if char.lower() == 'q':
                return True
        return False

    def _run_evaluation(self):
        rewards = []
        final_states = []
        wins = 0
        for _ in range(5):
            obs, _ = self.eval_env.reset()
            done = False
            total_reward = 0
            steps = 0
            while not done and steps < 1000:
                action_masks = self.eval_env.action_masks()
                action, _ = self.model.predict(obs, action_masks=action_masks, deterministic=True)
                obs, reward, done, _, _ = self.eval_env.step(action)
                total_reward += reward
                steps += 1
            
            fs = self.eval_env.state
            if fs['phase'] == 'Victory':
                wins += 1
            
            rewards.append(total_reward)
            total_boss_hp = get_total_boss_hp(fs)
            final_states.append(f"H={fs['hull_integrity']}, B={total_boss_hp}, R={fs['turn_count']}")
        
        mean_reward = sum(rewards) / len(rewards)
        win_rate = (wins / 5) * 100
        
        with self.lock:
            self.eval_history.append((self.n_calls, mean_reward, win_rate, final_states[0]))
        
        if not self.is_tty:
            self.console.print(
                f"📊 [bold yellow]Eval at {self.n_calls:,}:[/bold yellow] "
                f"Mean Reward: {mean_reward:+.2f} | "
                f"Win Rate: {win_rate:3.0f}% | "
                f"Sample: {final_states[0]}"
            )
        
        if mean_reward > self.best_mean_reward:
            if not self.is_tty:
                self.console.print(f"✨ [bold green]New Best Model![/bold green] ({mean_reward:.2f})")
            self.best_mean_reward = mean_reward
            self.model.save("solver/rl/models/ppo_sint_best")

    def _on_training_start(self) -> None:
        if self.is_tty:
            self.old_settings = termios.tcgetattr(sys.stdin)
            tty.setcbreak(sys.stdin.fileno())
            
            # Restore terminal on signals and SAVE
            def signal_handler(sig, frame):
                self._restore_terminal()
                self.console.print("\n[bold yellow]Interrupt received. Saving model...[/bold yellow]")
                self._stop_training = True
            
            signal.signal(signal.SIGINT, signal_handler)
            signal.signal(signal.SIGTERM, signal_handler)

            # Pre-populate layout
            self._update_live()
            self.live = Live(self.layout, console=self.console, refresh_per_second=4, screen=True)
            self.live.start()
        else:
            self.console.print("🚀 Starting training (Non-TTY mode)")
            
            def signal_handler(sig, frame):
                self.console.print("\n[bold yellow]Interrupt received. Stopping...[/bold yellow]")
                self._stop_training = True
                
            signal.signal(signal.SIGINT, signal_handler)
            signal.signal(signal.SIGTERM, signal_handler)
        
        self._stop_training = False
        self.stats["last_eval_stats"] = "Running baseline evaluation..."
        self._update_live()
        self._run_evaluation()
        self._update_live()

    def _restore_terminal(self):
        if self.live:
            self.live.stop()
            self.live = None
        if self.old_settings:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, self.old_settings)
            self.old_settings = None

    def _on_training_end(self) -> None:
        self._restore_terminal()
        self.console.print("✅ Training finished")

    def _on_step(self) -> bool:
        if self._stop_training or self._check_quit():
            self.console.print("\n[bold red]Stopping training...[/bold red]")
            return False

        # Capture latest reward and actions
        if hasattr(self, 'locals'):
            with self.lock:
                if 'rewards' in self.locals:
                    self.stats["latest_reward"] = float(np.mean(self.locals['rewards']))
                
                if 'actions' in self.locals:
                    self._record_action(self.locals['actions'])

        # Update latest state/trajectory from training_env (live progress)
        if self.training_env is not None:
            try:
                # SB3 VecEnvs allow retrieving attributes from sub-environments
                states = self.training_env.get_attr("state")
                histories = self.training_env.get_attr("history")
                
                if states and states[0]:
                    s = states[0]
                    with self.lock:
                        self.stats["latest_round"] = s.get("turn_count", 0)
                        
                        # State Summary
                        total_boss_hp = get_total_boss_hp(s)
                        self.stats["latest_state_summary"] = (
                            f"Hull: {s['hull_integrity']} | "
                            f"Total Boss HP: {total_boss_hp} | "
                            f"Phase: {s['phase']} | "
                            f"Active Players: {sum(1 for p in s['players'].values() if p['ap'] > 0)}"
                        )

                if histories and histories[0]:
                    # History is now a flat list of (player, action)
                    flat_history = histories[0]
                    total_traj_steps = len(flat_history)
                    last_12 = flat_history[-12:]
                    
                    log_lines = []
                    for p, a in last_12:
                        log_lines.append(f"  {p}: {a}")
                    
                    with self.lock:
                        self.stats["latest_trajectory"] = "\n".join(log_lines)
                        self.stats["total_trajectory_steps"] = total_traj_steps
            except Exception:
                # Attributes might not be ready in the first few calls
                pass

        if self.n_calls % 100 == 0:
            self._update_live()

        if self.n_calls % self.eval_freq == 0:
            self._run_evaluation()
            self._update_live()
            
        return True

    def _record_action(self, act):
        # Handle collections recursively
        if isinstance(act, (list, np.ndarray)):
            for a in act:
                self._record_action(a)
            return

        # In our SintEnv, we can map action index to a name
        # 0-9: Move, 10: Interact, 11: Bake, 12: Shoot, ...
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

    # Create environment
    def make_env():
        return SintEnv(num_players=args.num_players)

    env = make_vec_env(make_env, n_envs=8)

    # Load existing model if it exists, otherwise create a new one
    model_path = args.output
    if not model_path.endswith(".zip"):
        model_path += ".zip"

    if os.path.exists(model_path):
        print(f"📦 Loading existing model from {model_path}")
        model = MaskablePPO.load(
            model_path,
            env=env,
            tensorboard_log="solver/rl/logs/",
        )
    else:
        print("🆕 Creating new model")
        model = MaskablePPO(
            "MlpPolicy",
            env,
            verbose=1,
            tensorboard_log="solver/rl/logs/",
            learning_rate=3e-4,
            n_steps=4096,
            batch_size=256,
            n_epochs=10,
            gamma=0.99,
            gae_lambda=0.95,
            clip_range=0.2,
            ent_coef=0.01,
        )

    # Setup callbacks
    eval_env = SintEnv(num_players=args.num_players)
    tui_callback = TUICallback(eval_env, eval_freq=5000)
    checkpoint_callback = CheckpointCallback(
        save_freq=20000,
        save_path="solver/rl/models/",
        name_prefix="ppo_sint_checkpoint"
    )

    try:
        model.learn(total_timesteps=args.steps, callback=[checkpoint_callback, tui_callback])
    except KeyboardInterrupt:
        print("\nInterrupted by user")
    finally:
        # Save final model
        model.save(args.output)
        print(f"✅ Model saved to {args.output}")

if __name__ == "__main__":
    main()
