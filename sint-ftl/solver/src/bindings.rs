#[cfg(feature = "python")]
use crate::verification::run_verification;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;
#[cfg(feature = "python")]
use pythonize::{depythonize, pythonize};
#[cfg(feature = "python")]
use sint_core::types::{GameAction, GameState};

#[cfg(feature = "python")]
#[pyfunction]
fn verify_solution(
    py: Python,
    initial_state_dict: Bound<'_, PyAny>,
    rounds_list: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let initial_state: GameState = depythonize(&initial_state_dict).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid initial state: {}", e))
    })?;

    let rounds: Vec<Vec<(String, GameAction)>> = depythonize(&rounds_list).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid rounds list: {}", e))
    })?;

    let result = run_verification(initial_state, rounds);

    let py_result = pythonize(py, &result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    if !result.success {
        if let Some(summary) = result.failure_summary() {
            // Try to set "failure_summary" field on the resulting dict
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

    // Borrow history for calculate_score: &[&(PlayerId, GameAction)]
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
