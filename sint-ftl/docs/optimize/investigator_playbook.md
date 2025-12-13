# Investigator Playbook (Sub-Agent)

This playbook defines how to utilize the `codebase_investigator` tool to analyze solver trajectory files (`.txt` logs) and game rules to generate actionable optimization insights.

## Objective
To act as a forensic analyst and strategy coach for the solver. The investigator must read the raw simulation log, compare it against the game rules and current scoring weights, and identify **why** the solver is failing or performing sub-optimally.

## Context Requirements
When invoking the `codebase_investigator`, ensure the following are included in the analysis scope:
1.  **The Trajectory File:** `trajectories/solve_{seed}.txt` (e.g., `trajectories/solve_12345.txt`) - The evidence.
2.  **The Rules:** `docs/rules.md` - The high-level laws.
3.  **The Code:** `core/src/logic/` (recursively) - The absolute truth of the game physics.
4.  **The Scoring:** `solver/src/scoring/beam.rs` - The "brain" of the AI.

## Analysis Checklist
The investigator should answer these specific questions:

### 1. Cause of Death / Failure
*   **Hull Depletion:** Did we die by attrition (slow bleed) or a spike (boss attack)?
*   **Stalling:** Did the beam die because it reached the step limit?
    *   *Check:* Is the Round number increasing?
    *   *Check:* Are players repeating actions (oscillating)?
    *   *Check:* Is the solver refusing to end the turn (Pass/VoteReady) to avoid a penalty?
*   **Resource Starvation:** Did we run out of ammo? Were cannons empty while the boss fired?

### 2. Behavioral Patterns
*   **Role Adherence:** Are "Gunners" actually shooting? Are "Mechanics" repairing?
*   **Threat Response:** When a Fireball was telegraphed, did players move out of the room? Did they try to extinguish?
*   **Economy:** Did players pick up ammo efficiently, or did they ignore it?

### 3. Scoring Anomalies
*   **Gaming the System:** Is the solver doing useless actions (like picking up/dropping items) to farm a specific reward?
*   **Linearity Failure:** Does a flat penalty fail to capture escalating danger? (e.g., Is 1 Fire treated as manageable, but 3 Fires ignored until it's too late?)
*   **Fear of Progression:** Is `turn_penalty` or `threat_penalty` so high that the solver prefers to waste AP rather than advance the phase?

## Standard Prompt Template
Use this template when calling the `codebase_investigator` tool:

```text
Objective: Analyze the solver trajectories in the provided files (e.g., 'trajectories/solve_*.txt') to determine why the solver failed or stalled, and to compare successful vs. failed runs.

Context:
1. Review 'docs/rules.md' for high-level rules.
2. Search/Read 'core/src/logic/' to understand exact mechanics (e.g., how 'Fire' really spreads, what 'Card X' actually does in code).
3. Review 'solver/src/scoring/beam.rs' to understand current AI motivation.
4. Analyze ALL provided trajectory files (focusing on the last 100 steps of failed runs and key differences in strategy compared to successful runs).

Specific Questions:
1. What was the exact cause of termination for each failed run (Death, Beam Died, Step Limit)?
2. If "Beam Died": Look for stalling. Are players refusing to Pass/VoteReady? Is the Round count stuck?
3. If Death: Was it preventable? Did the solver ignore a Hazard or Enemy Telegraph?
4. Evaluate Scoring Linearity: Identify areas where flat weights failed (e.g., low hull should have infinite value, 3+ fires should be panic mode).
5. Provide 3 specific recommendations for adjusting 'BeamScoringWeights' or 'search.rs' logic, including math curves if needed.
```
