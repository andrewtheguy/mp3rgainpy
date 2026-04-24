pub use mp3rgain::aac::{
    analyze_aac_gains, apply_aac_gain, apply_aac_gain_with_undo, undo_aac_gain, AacAnalysis,
    AacGainLocation,
};
pub use mp3rgain::analysis::{analyze as analyze_mp3, ChannelMode, Mp3Analysis, MpegVersion};
pub use mp3rgain::gain::{
    apply_gain as apply_mp3_gain, undo_gain as undo_mp3_gain, GainOptions, GAIN_STEP_DB,
};
pub use mp3rgain::mp4meta::{is_aac_file, is_mp4_file};

#[cfg(feature = "python")]
mod python;
