pub use mp3rgain::aac::{
    analyze_aac_gains, apply_aac_gain, apply_aac_gain_with_undo, undo_aac_gain, AacAnalysis,
    AacGainLocation,
};
pub use mp3rgain::gain::GAIN_STEP_DB;
pub use mp3rgain::mp4meta::{is_aac_file, is_mp4_file};

#[cfg(feature = "python")]
mod python;
