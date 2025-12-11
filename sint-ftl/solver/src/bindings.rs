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
#[pymodule]
fn sint_solver(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(verify_solution, m)?)?;
    m.add_function(wrap_pyfunction!(get_trajectory_log, m)?)?;
    Ok(())
}
