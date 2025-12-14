import argparse
import os
import time
import sys
import select
import tty
import termios
import signal
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
        
        table.add_row("Total Steps", f"{self.n_calls:,}")
        table.add_row("Elapsed Time", f"{elapsed:.1f}s")
        table.add_row("Steps/sec (FPS)", f"{fps:.1f}")
        table.add_row("Current Round", f"{self.stats['latest_round']}")
        best_reward_str = f"{self.best_mean_reward:.2f}" if self.best_mean_reward != -float("inf") else "N/A"
        table.add_row("Best Mean Reward", best_reward_str)
        
        return Panel(table, title="[bold]Training Stats[/bold]", border_style="green")

    def _get_eval_panel(self):
        table = Table(box=box.SIMPLE, expand=True)
        table.add_column("Last Evaluation", style="yellow")
        table.add_row(self.stats["last_eval_stats"])
        return Panel(table, title="[bold]Evaluation Results[/bold]", border_style="yellow")

    def _get_io_panel(self):
        grid = Table.grid(expand=True)
        grid.add_column(style="cyan")
        total_steps = self.stats.get("total_trajectory_steps", 0)
        grid.add_row(f"[bold]Latest Trajectory (Last 12 of {total_steps} steps):[/bold]")
        grid.add_row(self.stats["latest_trajectory"])
        grid.add_row("")
        grid.add_row("[bold]Latest State Summary:[/bold]")
        grid.add_row(self.stats["latest_state_summary"])
        return Panel(grid, title="[bold]Last Input/Output[/bold]", border_style="magenta")

    def _update_live(self):
        if self.is_tty:
            self.layout["header"].update(self._get_header())
            self.layout["left"].update(self._get_stats_panel())
            self.layout["right"]["eval"].update(self._get_eval_panel())
            self.layout["right"]["io"].update(self._get_io_panel())
            self.layout["footer"].update(Panel(f"Training in progress... Step {self.n_calls:,} | Press 'q' to quit", border_style="white"))
        elif not self.is_tty and self.n_calls % 1000 == 0:
            elapsed = time.time() - self.start_time
            fps = self.n_calls / elapsed if elapsed > 0 else 0
            self.console.print(f"[{datetime.now().strftime('%H:%M:%S')}] Step: {self.n_calls:,} | FPS: {fps:.1f} | Best: {self.best_mean_reward:.2f}")

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
        final_stats = []
        for _ in range(3):
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
            rewards.append(total_reward)
            fs = self.eval_env.state
            final_stats.append(f"Hull={fs['hull_integrity']}, BossHP={fs['enemy']['hp']}, Round={fs['turn_count']}")
        
        mean_reward = sum(rewards) / len(rewards)
        eval_info = f"Mean Reward: {mean_reward:.2f} | Sample: {final_stats[0]}"
        self.stats["last_eval_stats"] = eval_info
        
        if not self.is_tty:
            self.console.print(f"📊 [bold yellow]Eval at {self.n_calls:,}:[/bold yellow] {eval_info}")
        
        if mean_reward > self.best_mean_reward:
            if not self.is_tty:
                self.console.print(f"✨ [bold green]New Best Model![/bold green] ({mean_reward:.2f})")
            self.best_mean_reward = mean_reward
            self.model.save("solver/rl/models/ppo_sint_best")

    def _on_training_start(self) -> None:
        if self.is_tty:
            self.old_settings = termios.tcgetattr(sys.stdin)
            tty.setcbreak(sys.stdin.fileno())
            
            # Restore terminal on signals
            def signal_handler(sig, frame):
                self._restore_terminal()
                sys.exit(0)
            
            signal.signal(signal.SIGINT, signal_handler)
            signal.signal(signal.SIGTERM, signal_handler)

            # Pre-populate layout
            self._update_live()
            self.live = Live(self.layout, console=self.console, refresh_per_second=4, screen=True)
            self.live.start()
        else:
            self.console.print("🚀 Starting training (Non-TTY mode)")
        
        # Mark as needing initial evaluation
        self.stats["last_eval_stats"] = "Waiting for first step..."
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
        if self._check_quit():
            self.console.print("\n[bold red]Stopping training...[/bold red]")
            return False

        # Update latest state/trajectory from training_env (live progress)
        if self.training_env is not None:
            try:
                # SB3 VecEnvs allow retrieving attributes from sub-environments
                states = self.training_env.get_attr("state")
                histories = self.training_env.get_attr("history")
                
                if states and states[0]:
                    s = states[0]
                    self.stats["latest_round"] = s.get("turn_count", 0)
                    
                    # State Summary
                    self.stats["latest_state_summary"] = (
                        f"Hull: {s['hull_integrity']} | "
                        f"Boss HP: {s['enemy']['hp']} | "
                        f"Phase: {s['phase']} | "
                        f"Active Players: {sum(1 for p in s['players'].values() if p['ap'] > 0)}"
                    )

                if histories and histories[0]:
                    # Flatten history to get a continuous list of (player, action)
                    flat_history = [item for sublist in histories[0] for item in sublist]
                    total_traj_steps = len(flat_history)
                    last_12 = flat_history[-12:]
                    
                    log_lines = []
                    for p, a in last_12:
                        log_lines.append(f"  {p}: {a}")
                    
                    self.stats["latest_trajectory"] = "\n".join(log_lines)
                    self.stats["total_trajectory_steps"] = total_traj_steps
            except Exception:
                # Attributes might not be ready in the first few calls
                pass

        # Run initial evaluation on the first step
        if self.n_calls == 0:
            self.stats["last_eval_stats"] = "Running baseline evaluation..."
            self._update_live()
            self._run_evaluation()
            self._update_live()

        if self.n_calls % 100 == 0:
            self._update_live()

        if self.n_calls % self.eval_freq == 0:
            self._run_evaluation()
            self._update_live()
            
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

    # Setup callbacks
    eval_env = SintEnv(num_players=args.num_players)
    tui_callback = TUICallback(eval_env, eval_freq=5000)
    checkpoint_callback = CheckpointCallback(
        save_freq=20000,
        save_path="solver/rl/models/",
        name_prefix="ppo_sint_checkpoint"
    )

    model.learn(total_timesteps=args.steps, callback=[checkpoint_callback, tui_callback])

    # Save final model
    model.save(args.output)
    print(f"✅ Model saved to {args.output}")

if __name__ == "__main__":
    main()
