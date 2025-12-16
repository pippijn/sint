import time
import numpy as np
from datetime import datetime
from rich.console import Console
from rich.layout import Layout
from rich.panel import Panel
from rich.live import Live
from rich.table import Table
from rich import box

def get_total_boss_hp(s):
    if not s or not isinstance(s, dict):
        return 50
    # Boss HPs from core/src/logic.rs
    boss_hps = [5, 10, 15, 20]
    level = s.get('boss_level', 0)
    if s.get('phase') == 'Victory':
        return 0
    
    # Current boss HP + all future bosses
    enemy = s.get('enemy', {})
    total = enemy.get('hp', 0)
    for i in range(level + 1, len(boss_hps)):
        total += boss_hps[i]
    return total

class SintDashboard:
    def __init__(self, start_time):
        self.console = Console()
        self.start_time = start_time
        self.layout = self._setup_layout()
        self.live = None

    def _setup_layout(self):
        layout = Layout()
        layout.split_column(
            Layout(name="header", size=3),
            Layout(name="main", ratio=1),
            Layout(name="footer", size=3),
        )
        layout["main"].split_row(
            Layout(name="left", ratio=1),
            Layout(name="best", ratio=2),
            Layout(name="latest", ratio=2),
        )
        layout["left"].split_column(
            Layout(name="stats", size=15),
            Layout(name="actions", ratio=1),
            Layout(name="eval", ratio=1),
        )
        return layout

    def start(self):
        self.live = Live(self.layout, console=self.console, refresh_per_second=4, screen=True)
        self.live.start()

    def stop(self):
        if self.live:
            self.live.stop()
            self.live = None

    def update(self, stats, best_ep, eval_history):
        """Update the TUI layout with the latest statistics."""
        self.layout["header"].update(self._get_header())
        self.layout["left"]["stats"].update(self._get_stats_panel(stats, eval_history))
        self.layout["left"]["actions"].update(self._get_actions_panel(stats))
        self.layout["left"]["eval"].update(self._get_eval_panel(eval_history))
        
        # BEST Episode Panel
        self.layout["best"].update(self._get_episode_panel(
            "HALL OF FAME (Best)", 
            best_ep["reward"],
            best_ep["breakdown"],
            best_ep["trajectory"],
            best_ep["summary"],
            best_ep["steps"],
            best_ep["rounds"],
            best_ep["seed"],
            "magenta"
        ))
        
        # LATEST Episode Panel
        self.layout["latest"].update(self._get_episode_panel(
            "REAL-TIME (Latest)",
            stats["current_reward"],
            stats["latest_reward_breakdown"],
            stats["latest_trajectory"],
            stats["latest_state_summary"],
            stats["latest_trajectory_steps"],
            stats["latest_round"],
            stats["latest_seed"],
            "cyan"
        ))
        
        self.layout["footer"].update(Panel(f"Training in progress... Step {stats['total_steps']:,} | Press 'q' to quit", border_style="white"))

    def _get_header(self):
        grid = Table.grid(expand=True)
        grid.add_column(justify="left", ratio=1)
        grid.add_column(justify="right", ratio=1)
        grid.add_row(
            "[bold white]🚢 Sint-FTL RL Training[/bold white]",
            f"[bold blue]{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}[/bold blue]"
        )
        return Panel(grid, style="white on blue")

    def _get_stats_panel(self, stats, eval_history):
        table = Table(box=box.SIMPLE, expand=True)
        table.add_column("Metric", style="cyan")
        table.add_column("Value", style="magenta")
        
        elapsed = time.time() - self.start_time
        fps = stats["total_steps"] / elapsed if elapsed > 0 else 0
        
        table.add_row("Total Steps", f"{stats['total_steps']:,}")
        table.add_row("Elapsed Time", f"{elapsed:.1f}s")
        table.add_row("Steps/sec (FPS)", f"{fps:.1f}")
        best_reward_str = f"{stats['best_mean_reward']:.2f}" if stats['best_mean_reward'] != -float("inf") else "N/A"
        table.add_row("Best Mean Reward", best_reward_str)

        total_ep = stats.get("total_episodes", 0)
        table.add_row("Total Episodes", f"{total_ep:,}")
        if total_ep > 0:
            win_rate = (stats.get("wins", 0) / total_ep) * 100
            loss_rate = (stats.get("losses", 0) / total_ep) * 100
            timeout_rate = (stats.get("timeouts", 0) / total_ep) * 100
            table.add_row("Win Rate", f"[green]{win_rate:5.1f}%[/green]")
            table.add_row("Loss Rate", f"[red]{loss_rate:5.1f}%[/red]")
            table.add_row("Timeout Rate", f"[yellow]{timeout_rate:5.1f}%[/yellow]")

        if eval_history:
            history_str = " ".join([f"{r:+.1f}" for _, r, _, _ in eval_history[-8:]])
            table.add_row("Eval Trend", history_str)

        return Panel(table, title="[bold]Training Stats[/bold]", border_style="green")

    def _get_actions_panel(self, stats):
        table = Table(box=box.SIMPLE, expand=True)
        table.add_column("Action", style="cyan", width=12)
        table.add_column("Percent", style="magenta", width=8)
        table.add_column("Graph", style="white")

        all_action_names = [
            "Move", "Shoot", "PickUp", "Drop", "Shields", "Evasive", 
            "Extinguish", "Repair", "Bake", "Interact", "FirstAid", 
            "Revive", "Throw", "Pass", "Other"
        ]

        total_actions = sum(stats["action_counts"].values())
        action_data = []
        for act_name in all_action_names:
            count = stats["action_counts"].get(act_name, 0)
            pct = (count / total_actions * 100) if total_actions > 0 else 0
            action_data.append((act_name, pct))
        
        action_data.sort(key=lambda x: x[1], reverse=True)

        for act_name, pct in action_data:
            bar_len = 10
            filled = int(pct / (100 / bar_len))
            bar = "█" * filled + "░" * (bar_len - filled)
            color = "green" if pct > 0 else "dim white"
            table.add_row(act_name, f"{pct:5.1f}%", f"[{color}]{bar}[/]")
        
        return Panel(table, title="[bold]Action Distribution[/bold]", border_style="cyan")

    def _get_eval_panel(self, eval_history):
        table = Table(box=box.SIMPLE, expand=True)
        table.add_column("Step", style="cyan", width=10)
        table.add_column("Mean Rew", style="magenta")
        table.add_column("Win %", style="green")
        table.add_column("Final State", style="white")
        
        for step, reward, win_rate, fs in eval_history[-5:]:
            table.add_row(f"{step:,}", f"{reward:+.2f}", f"{win_rate:3.0f}%", fs)
            
        return Panel(table, title="[bold]Evaluation History[/bold]", border_style="yellow")

    def _get_episode_panel(self, title, reward, breakdown, trajectory, summary, steps, rounds, seed, border_style):
        grid = Table.grid(expand=True)
        grid.add_column(ratio=1)
        
        rew_table = Table(box=box.SIMPLE, expand=True, title="[bold]Reward Breakdown[/bold]")
        rew_table.add_column("Component", style="cyan")
        rew_table.add_column("Value", style="magenta")
        rew_table.add_column("Graph", style="white")

        if not breakdown:
            rew_table.add_row("Waiting...", "", "")
        else:
            for key, value in breakdown.items():
                if key == "total": continue 
                bar_len = 10
                mag = abs(value)
                filled = min(bar_len, int(mag / 100)) if mag > 10 else int(mag / 10)
                bar = "█" * filled + "░" * (bar_len - filled)
                color = "green" if value > 0 else "red" if value < 0 else "dim white"
                rew_table.add_row(key.capitalize(), f"{value:+.2f}", f"[{color}]{bar}[/]")
        
        grid.add_row(rew_table)
        grid.add_row("")
        
        stats_table = Table(box=box.MINIMAL, expand=True)
        stats_table.add_column("Metric", style="cyan")
        stats_table.add_column("Value", style="white")
        stats_table.add_row("Game Seed", str(seed))
        stats_table.add_row("Total Rounds", str(rounds))
        stats_table.add_row("Total Steps", str(steps))
        stats_table.add_row("Final State", summary)
        grid.add_row(Panel(stats_table, title="[bold]Episode Summary[/bold]", border_style="green"))

        grid.add_row("")
        grid.add_row(Panel(trajectory, title="[bold]Last 20 Actions[/bold]", border_style="dim"))
        
        return Panel(grid, title=f"[bold]{title} (Total Rew: {reward:+.1f})[/bold]", border_style=border_style)