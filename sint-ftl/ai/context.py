from dataclasses import dataclass, field
from typing import List, Optional
import time

@dataclass
class LogEntry:
    timestamp: float
    text: str
    is_summary: bool = False

@dataclass
class MemoryBank:
    persistent_summaries: List[str] = field(default_factory=list)
    recent_logs: List[LogEntry] = field(default_factory=list)
    max_log_size: int = 50
    summarize_chunk_size: int = 25

    def add_log(self, text: str) -> None:
        self.recent_logs.append(LogEntry(time.time(), text))

    def should_summarize(self) -> bool:
        return len(self.recent_logs) >= self.max_log_size

    def get_chunk_to_summarize(self) -> List[str]:
        # Return text of oldest chunk
        chunk = self.recent_logs[:self.summarize_chunk_size]
        return [entry.text for entry in chunk]

    def commit_summary(self, summary_text: str) -> None:
        self.persistent_summaries.append(summary_text)
        # Remove the chunk we just summarized
        self.recent_logs = self.recent_logs[self.summarize_chunk_size:]

    def get_full_context_text(self) -> str:
        """Combines summaries and recent logs into a single context string."""
        context = []
        if self.persistent_summaries:
            context.append("PREVIOUSLY ON OPERATION PEPPERNUT:")
            context.extend(self.persistent_summaries)
            context.append("\nRECENT EVENTS:")
        
        for entry in self.recent_logs:
            context.append(entry.text)
            
        return "\n".join(context)
