use std::path::Path;

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyList};

use crate::{
    analyze_aac_gains, analyze_mp3, apply_aac_gain, apply_aac_gain_with_undo, apply_mp3_gain,
    is_aac_file, is_mp4_file, undo_aac_gain, undo_mp3_gain, ChannelMode, GainOptions, MpegVersion,
    GAIN_STEP_DB,
};

#[pyclass(frozen)]
#[derive(Clone)]
struct AacGainLocation {
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
struct AacGainAnalysis {
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
impl AacGainAnalysis {
    #[getter]
    fn gain_locations<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let locs: Vec<Py<AacGainLocation>> = self
            .locations
            .iter()
            .map(|&(si, fo, sbo, bo, ch, og)| {
                Py::new(
                    py,
                    AacGainLocation {
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

#[pyclass(frozen)]
struct Mp3Analysis {
    #[pyo3(get)]
    frame_count: usize,
    #[pyo3(get)]
    mpeg_version: String,
    #[pyo3(get)]
    channel_mode: String,
    #[pyo3(get)]
    channel_count: usize,
    #[pyo3(get)]
    min_gain: u8,
    #[pyo3(get)]
    max_gain: u8,
    #[pyo3(get)]
    avg_gain: f64,
    #[pyo3(get)]
    headroom_steps: i32,
    #[pyo3(get)]
    headroom_db: f64,
}

fn mpeg_version_str(v: MpegVersion) -> String {
    v.as_str().to_string()
}

fn channel_mode_str(m: ChannelMode) -> String {
    m.as_str().to_string()
}

#[pyfunction]
fn aac_apply_gain(file_path: &str, gain_steps: i32) -> PyResult<usize> {
    apply_aac_gain(Path::new(file_path), gain_steps)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn aac_apply_gain_with_undo(file_path: &str, gain_steps: i32) -> PyResult<usize> {
    apply_aac_gain_with_undo(Path::new(file_path), gain_steps)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn aac_undo_gain(file_path: &str) -> PyResult<usize> {
    undo_aac_gain(Path::new(file_path)).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn aac_analyze(file_path: &str) -> PyResult<AacGainAnalysis> {
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

    Ok(AacGainAnalysis {
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
fn mp3_apply_gain(file_path: &str, gain_steps: i32) -> PyResult<usize> {
    apply_mp3_gain(Path::new(file_path), gain_steps)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn mp3_apply_gain_with_undo(file_path: &str, gain_steps: i32) -> PyResult<usize> {
    GainOptions::new(gain_steps)
        .undo(true)
        .apply(Path::new(file_path))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn mp3_undo_gain(file_path: &str) -> PyResult<usize> {
    undo_mp3_gain(Path::new(file_path)).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn mp3_analyze(file_path: &str) -> PyResult<Mp3Analysis> {
    let analysis =
        analyze_mp3(Path::new(file_path)).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let channel_mode = analysis.channel_mode();
    Ok(Mp3Analysis {
        frame_count: analysis.frame_count(),
        mpeg_version: mpeg_version_str(analysis.mpeg_version()),
        channel_mode: channel_mode_str(channel_mode),
        channel_count: channel_mode.channel_count(),
        min_gain: analysis.min_gain(),
        max_gain: analysis.max_gain(),
        avg_gain: analysis.avg_gain(),
        headroom_steps: analysis.headroom_steps(),
        headroom_db: analysis.headroom_db(),
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
#[pyo3(name = "_native")]
fn native_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "__all__",
        vec![
            "aac_apply_gain",
            "aac_apply_gain_with_undo",
            "aac_undo_gain",
            "aac_analyze",
            "mp3_apply_gain",
            "mp3_apply_gain_with_undo",
            "mp3_undo_gain",
            "mp3_analyze",
            "is_mp4_file",
            "is_aac_file",
            "AacGainLocation",
            "AacGainAnalysis",
            "Mp3Analysis",
            "GAIN_STEP_DB",
        ],
    )?;
    module.add("GAIN_STEP_DB", GAIN_STEP_DB)?;
    module.add_class::<AacGainLocation>()?;
    module.add_class::<AacGainAnalysis>()?;
    module.add_class::<Mp3Analysis>()?;
    module.add_function(wrap_pyfunction!(aac_apply_gain, module)?)?;
    module.add_function(wrap_pyfunction!(aac_apply_gain_with_undo, module)?)?;
    module.add_function(wrap_pyfunction!(aac_undo_gain, module)?)?;
    module.add_function(wrap_pyfunction!(aac_analyze, module)?)?;
    module.add_function(wrap_pyfunction!(mp3_apply_gain, module)?)?;
    module.add_function(wrap_pyfunction!(mp3_apply_gain_with_undo, module)?)?;
    module.add_function(wrap_pyfunction!(mp3_undo_gain, module)?)?;
    module.add_function(wrap_pyfunction!(mp3_analyze, module)?)?;
    module.add_function(wrap_pyfunction!(is_mp4_file_py, module)?)?;
    module.add_function(wrap_pyfunction!(is_aac_file_py, module)?)?;
    Ok(())
}
