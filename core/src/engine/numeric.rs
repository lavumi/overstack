pub(crate) const STATUS_TICK_THRESHOLD: f32 = 100.0;
pub(crate) const STATUS_TICK_RATE: f32 = 100.0;

pub(crate) fn round_hp(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

pub(crate) fn status_duration_display_secs(duration: f32) -> u32 {
    duration.max(0.0).round() as u32
}
