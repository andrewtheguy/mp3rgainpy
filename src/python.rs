use std::path::Path;

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyList};

use crate::{
    analyze_aac_gains, apply_aac_gain, apply_aac_gain_with_undo, is_aac_file, is_mp4_file,
    undo_aac_gain, GAIN_STEP_DB,
};

#[pyclass(frozen)]
#[derive(Clone)]
struct GainLocation {
    #[pyo3(get)]
    sample_index: u32,
    #[pyo3(get)]
    file_offset: u64,
    #[pyo3(get)]
    sample_byte_offset: u32,
    #[pyo3(get)]
    bit_offset: u8,
    #[pyo3(get)]
    channel: u8,
    #[pyo3(get)]
    original_gain: u8,
}

#[pyclass(frozen)]
struct GainAnalysis {
    #[pyo3(get)]
    sample_count: u32,
    #[pyo3(get)]
    channel_count: u8,
    #[pyo3(get)]
    min_gain: u8,
    #[pyo3(get)]
    max_gain: u8,
    #[pyo3(get)]
    sample_rate: u32,
    #[pyo3(get)]
    parse_warnings: u32,
    locations: Vec<(u32, u64, u32, u8, u8, u8)>,
}

#[pymethods]
impl GainAnalysis {
    #[getter]
    fn gain_locations<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let locs: Vec<Py<GainLocation>> = self
            .locations
            .iter()
            .map(|&(si, fo, sbo, bo, ch, og)| {
                Py::new(
                    py,
                    GainLocation {
                        sample_index: si,
                        file_offset: fo,
                        sample_byte_offset: sbo,
                        bit_offset: bo,
                        channel: ch,
                        original_gain: og,
                    },
                )
            })
            .collect::<PyResult<_>>()?;
        Ok(PyList::new(py, locs)?)
    }
}

#[pyfunction]
fn apply_gain(file_path: &str, gain_steps: i32) -> PyResult<usize> {
    apply_aac_gain(Path::new(file_path), gain_steps)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn apply_gain_with_undo(file_path: &str, gain_steps: i32) -> PyResult<usize> {
    apply_aac_gain_with_undo(Path::new(file_path), gain_steps)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn undo_gain(file_path: &str) -> PyResult<usize> {
    undo_aac_gain(Path::new(file_path)).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn analyze(file_path: &str) -> PyResult<GainAnalysis> {
    let analysis = analyze_aac_gains(Path::new(file_path))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let locations: Vec<(u32, u64, u32, u8, u8, u8)> = analysis
        .gain_locations()
        .iter()
        .map(|loc| {
            (
                loc.sample_index(),
                loc.file_offset(),
                loc.sample_byte_offset(),
                loc.bit_offset(),
                loc.channel(),
                loc.original_gain(),
            )
        })
        .collect();

    Ok(GainAnalysis {
        sample_count: analysis.sample_count(),
        channel_count: analysis.channel_count(),
        min_gain: analysis.min_gain(),
        max_gain: analysis.max_gain(),
        sample_rate: analysis.sample_rate(),
        parse_warnings: analysis.parse_warnings(),
        locations,
    })
}

#[pyfunction]
#[pyo3(name = "is_mp4_file")]
fn is_mp4_file_py(file_path: &str) -> bool {
    is_mp4_file(Path::new(file_path))
}

#[pyfunction]
#[pyo3(name = "is_aac_file")]
fn is_aac_file_py(file_path: &str) -> bool {
    is_aac_file(Path::new(file_path))
}

#[pymodule]
#[pyo3(name = "m4again")]
fn m4again_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "__all__",
        vec![
            "apply_gain",
            "apply_gain_with_undo",
            "undo_gain",
            "analyze",
            "is_mp4_file",
            "is_aac_file",
            "GainLocation",
            "GainAnalysis",
            "GAIN_STEP_DB",
        ],
    )?;
    module.add("GAIN_STEP_DB", GAIN_STEP_DB)?;
    module.add_class::<GainLocation>()?;
    module.add_class::<GainAnalysis>()?;
    module.add_function(wrap_pyfunction!(apply_gain, module)?)?;
    module.add_function(wrap_pyfunction!(apply_gain_with_undo, module)?)?;
    module.add_function(wrap_pyfunction!(undo_gain, module)?)?;
    module.add_function(wrap_pyfunction!(analyze, module)?)?;
    module.add_function(wrap_pyfunction!(is_mp4_file_py, module)?)?;
    module.add_function(wrap_pyfunction!(is_aac_file_py, module)?)?;
    Ok(())
}
