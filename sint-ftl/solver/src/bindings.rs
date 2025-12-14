#[cfg(feature = "python")]
use crate::verification::run_verification;
use once_cell::sync::Lazy;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;
#[cfg(feature = "python")]
use pythonize::{depythonize, pythonize};
#[cfg(feature = "python")]
use sint_core::logic::GameLogic;
#[cfg(feature = "python")]
use sint_core::types::{GameAction, GameState};
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(feature = "python")]
struct SessionState {
    state: GameState,
    history: Vec<Vec<(String, GameAction)>>,
}

#[cfg(feature = "python")]
static SESSION_CACHE: Lazy<Mutex<HashMap<String, SessionState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(feature = "python")]
fn parse_rounds(rounds_list: &Bound<'_, PyAny>) -> PyResult<Vec<Vec<(String, GameAction)>>> {
    if let Ok(r) = depythonize(rounds_list) {
        Ok(r)
    } else {
        let raw_rounds: Vec<Vec<(String, String)>> = depythonize(rounds_list).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid rounds list: {}", e))
        })?;
        Ok(raw_rounds
            .into_iter()
            .map(|round| {
                round
                    .into_iter()
                    .map(|(pid, cmd)| (pid, crate::verification::parse_game_action(&cmd)))
                    .collect()
            })
            .collect())
    }
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (player_ids, seed, rounds_list, session_id=None))]
fn verify_solution(
    py: Python,
    player_ids: Vec<String>,
    seed: u64,
    rounds_list: Bound<'_, PyAny>,
    session_id: Option<String>,
) -> PyResult<Py<PyAny>> {
    let rounds = parse_rounds(&rounds_list)?;

    let initial_state = if let Some(sid) = &session_id {
        let mut cache = SESSION_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(sid) {
            // If the cached history is a prefix of the new rounds, we can resume
            if rounds.starts_with(&cached.history) {
                let state = cached.state.clone();
                let remaining_rounds = rounds[cached.history.len()..].to_vec();
                let result = run_verification(state.clone(), remaining_rounds);

                // Update cache with the new final state
                cache.insert(
                    sid.clone(),
                    SessionState {
                        state: result.final_state.clone(),
                        history: rounds.clone(),
                    },
                );

                let py_result = pythonize(py, &result).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;

                if !result.success {
                    if let Some(summary) = result.failure_summary() {
                        if let Ok(dict) = py_result.clone().cast_into::<PyDict>() {
                            let _ = dict.set_item("failure_summary", summary);
                        }
                    }
                }
                return Ok(py_result.into());
            }
        }
        // Fallback: Start from scratch and initialize cache
        let state = GameLogic::new_game(player_ids, seed);
        state
    } else {
        GameLogic::new_game(player_ids, seed)
    };

    let result = run_verification(initial_state, rounds.clone());

    if let Some(sid) = session_id {
        let mut cache = SESSION_CACHE.lock().unwrap();
        cache.insert(
            sid,
            SessionState {
                state: result.final_state.clone(),
                history: rounds,
            },
        );
    }

    let py_result = pythonize(py, &result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    if !result.success {
        if let Some(summary) = result.failure_summary() {
            if let Ok(dict) = py_result.clone().cast_into::<PyDict>() {
                let _ = dict.set_item("failure_summary", summary);
            }
        }
    }

    Ok(py_result.into())
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_trajectory_log(
    _py: Python,
    initial_state_dict: Bound<'_, PyAny>,
    history_list: Bound<'_, PyAny>,
) -> PyResult<Vec<String>> {
    let initial_state: GameState = depythonize(&initial_state_dict).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid initial state: {}", e))
    })?;

    let history: Vec<(String, GameAction)> = depythonize(&history_list).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid history list: {}", e))
    })?;

    Ok(crate::replay::format_trajectory(initial_state, history))
}

#[cfg(feature = "python")]
#[pyfunction]
fn compute_score(
    _py: Python,
    parent_dict: Bound<'_, PyAny>,
    current_dict: Bound<'_, PyAny>,
    history_list: Bound<'_, PyAny>,
) -> PyResult<f64> {
    let parent: GameState = depythonize(&parent_dict).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid parent state: {}", e))
    })?;

    let current: GameState = depythonize(&current_dict).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid current state: {}", e))
    })?;

    let history: Vec<(String, GameAction)> = depythonize(&history_list).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid history list: {}", e))
    })?;

    let borrowed_history: Vec<&(String, GameAction)> = history.iter().collect();

    let weights = crate::scoring::beam::BeamScoringWeights::default();
    let distances = sint_core::logic::pathfinding::MapDistances::new(&current.map);

    let details = crate::scoring::beam::calculate_score(
        &parent,
        &current,
        &borrowed_history,
        &weights,
        &distances,
    );

    Ok(details.total)
}

#[cfg(feature = "python")]
#[pyfunction]
fn compute_score_rhea(_py: Python, state_dict: Bound<'_, PyAny>) -> PyResult<f64> {
    let state: GameState = depythonize(&state_dict).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid game state: {}", e))
    })?;

    let weights = crate::scoring::rhea::RheaScoringWeights::default();
    let details = crate::scoring::rhea::score_rhea(&state, &weights);

    Ok(details.total)
}

#[cfg(feature = "python")]
#[pymodule]
fn sint_solver(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(verify_solution, m)?)?;
    m.add_function(wrap_pyfunction!(get_trajectory_log, m)?)?;
    m.add_function(wrap_pyfunction!(compute_score, m)?)?;
    m.add_function(wrap_pyfunction!(compute_score_rhea, m)?)?;
    Ok(())
}
