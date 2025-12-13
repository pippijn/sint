# Main Agent Optimization Playbook

This playbook outlines the workflow for the main agent to iteratively optimize the solver using the `codebase_investigator` as a feedback mechanism.

## Core Philosophy
**Do not read the full trajectory file or the game rules yourself.** This preserves your context window. Your role is **Orchestrator**; the `codebase_investigator` is the **Analyst**.

## Context & Safety Strategy
*   **Rules (`docs/rules.md`):** Do **NOT** read. The Investigator is explicitly instructed to review this. Trust their semantic understanding.
*   **Trajectory Logs (`*.txt`):** Do **NOT** read. Delegate forensic analysis to the Investigator.
*   **Source Code (`*.rs`):** Read **ONLY** the specific file you intend to modify, and **ONLY** immediately before applying a fix.
    *   *Why?* You need the exact string context for the `replace` tool to work safely. You do not need to understand the full system architecture; the Investigator will point you to the right location.

## Workflow Loop

### 1. Generate Data
Run the solver with sufficient steps. **Always invoke the script directly, do not use python3.**

**Single Seed:**
```bash
scripts/run_solve.py --steps 3000 --seeds 12345
```

**Multiple Seeds (Parallel):**
This verifies robustness across different RNG states.
```bash
scripts/run_solve.py --steps 3000 --seeds 12345,54321,99999 --parallel
```

*   *Check:* Did it finish? Did it win? If it won, can we optimize for speed (fewer rounds)?

### 2. Delegate Analysis
If the result is unsatisfactory (or if optimizing for speed), invoke the `codebase_investigator`.

*   **Tool:** `codebase_investigator`
*   **Prompt:** Use the "Standard Prompt Template" from `docs/optimize/investigator_playbook.md`, replacing `{TRAJECTORY_FILE}` with the actual filename (e.g., `solve_12345.txt`).

### 3. Interpret & Implement
Read the investigator's report. Look for:
*   **Weight Adjustments:** "Increase `fire_penalty` by 50%." -> Modify values in `solver/src/scoring/beam.rs`.
*   **Non-Linearity:** "Fire is fine at 1, but deadly at 3." -> **Implement curves** (e.g., `hazard_count.powf(weights.fire_exponent)`) instead of flat multipliers.
*   **New Heuristics:** "Solver misses X pattern." -> **Code new logic** in `score_state` to detect X and add fields to `BeamScoringWeights`.
*   **Rule:** **NO MAGIC NUMBERS**. All coefficients, thresholds, and exponents must be defined as fields in the `BeamScoringWeights` struct (in `solver/src/scoring/beam.rs`).
*   **Do NOT** hardcode values like `500.0` or `10.0` directly in `score_static`. Define a field like `my_new_heuristic_reward` in the struct, add it to `BeamScoringWeights::default()`, and use `weights.my_new_heuristic_reward`.
*   **Logic Gaps:** "Solver is stuck in Planning phase." -> Modify `solver/src/search.rs` (e.g., fallback logic, pruning).
*   **Rule Conflicts:** "Solver thinks it can Y but it can't." -> Check `core/src/logic`.

*   **Action:** Use `replace` to apply changes. You are authorized to write new code if tuning isn't enough.
    *   **CRITICAL:** Make **small, atomic edits**. Do not try to replace the entire file or large functions in one go. Use multiple smaller `replace` calls if necessary to ensure precision and avoid context errors.

### 4. Verify
*   Re-run the solver (Step 1).
*   Compare the summary output (Total Rounds, Bosses Defeated) to the previous run's summary (which you should see in your conversation history).

## Common Tuning Scenarios

### Scenario: "Beam Died" (Stalling)
*   **Symptom:** Solver runs for max steps but Round count is low (e.g., Step 2000, Round 12).
*   **Root Cause:** The penalty for advancing the turn (Turn Penalty + Pending Enemy Damage) is higher than the penalty for wasting AP in the current turn.
*   **Fix:**
    1.  Add/Increase `step_penalty` in `BeamScoringWeights` (penalize dithering).
    2.  Increase `ap_balance` (make keeping AP valuable).
    3.  Decrease `turn_penalty` (make advancing less scary).

### Scenario: Suicide / Glass Cannon
*   **Symptom:** Solver kills bosses fast but dies to Hull 0 or Fire.
*   **Root Cause:** `enemy_hp` reward or `shooting_reward` outweighs `hull_integrity` or `fire_penalty`.
*   **Fix:** Increase `hull_integrity` and `fire_penalty_base`.

### Scenario: Cowardice
*   **Symptom:** Solver survives 100 rounds but kills nothing.
*   **Root Cause:** `threat_penalty` is too high (afraid to be in target rooms) or `enemy_hp` reward is too low.
*   **Fix:** Increase `enemy_hp` reward. Reduce `threat_player_penalty`.
