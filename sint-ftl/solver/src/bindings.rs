#[cfg(feature = "python")]
use crate::verification::{parse_solution_text, run_verification};
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;
#[cfg(feature = "python")]
use pythonize::{depythonize, pythonize};
#[cfg(feature = "python")]
use sint_core::types::{Action, GameState};

#[cfg(feature = "python")]
#[pyfunction]
fn parse_solution(py: Python, text: String) -> PyResult<Py<PyAny>> {
    let (actions, seed, players) = parse_solution_text(&text);
    let res = (actions, seed, players);
    let py_res = pythonize(py, &res)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(py_res.into())
}

#[cfg(feature = "python")]
#[pyfunction]
fn verify_solution(
    py: Python,
    initial_state_dict: Bound<'_, PyAny>,
    actions_list: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let initial_state: GameState = depythonize(&initial_state_dict).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid initial state: {}", e))
    })?;

    let actions: Vec<(String, Action)> = depythonize(&actions_list).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid actions list: {}", e))
    })?;

    let result = run_verification(initial_state, actions);

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
#[pymodule]
fn sint_solver(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_solution, m)?)?;
    m.add_function(wrap_pyfunction!(verify_solution, m)?)?;
    Ok(())
}
