#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pythonize::{depythonize, pythonize};
#[cfg(feature = "python")]
use crate::{types::*, logic::GameLogic};

#[cfg(feature = "python")]
#[pyfunction]
fn new_game(py: Python, player_ids: Vec<String>, seed: u64) -> PyResult<Py<PyAny>> {
    let state = GameLogic::new_game(player_ids, seed);
    let py_state = pythonize(py, &state).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(py_state.into())
}

#[cfg(feature = "python")]
#[pyfunction]
fn apply_action_with_id(py: Python, state_dict: &Bound<'_, PyAny>, player_id: String, action_dict: &Bound<'_, PyAny>, seed: Option<u64>) -> PyResult<Py<PyAny>> {
    let state: GameState = depythonize(state_dict).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let action: Action = depythonize(action_dict).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    match GameLogic::apply_action(state, &player_id, action, seed) {
        Ok(new_state) => {
            let py_state = pythonize(py, &new_state).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(py_state.into())
        },
        Err(e) => {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
        }
    }
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_schema_json() -> PyResult<String> {
    let schema = schemars::schema_for!(Action);
    serde_json::to_string_pretty(&schema).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

#[cfg(feature = "python")]
#[pymodule]
fn sint_core(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new_game, m)?)?;
    m.add_function(wrap_pyfunction!(apply_action_with_id, m)?)?;
    m.add_function(wrap_pyfunction!(get_schema_json, m)?)?;
    Ok(())
}