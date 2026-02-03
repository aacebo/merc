/// Compute effective threshold based on text length.
/// Short text (<= 20 chars) gets a lower threshold (easier to accept).
/// Long text (> 200 chars) gets a higher threshold (harder to accept).
#[macro_export]
macro_rules! threshold {
    ($text:expr) => {
        match $text.len() {
            0..=20 => 0.75 - 0.05,
            21..=200 => 0.75,
            _ => 0.75 + 0.05,
        }
    };
    ($text:expr, threshold = $threshold:expr) => {
        match $text.len() {
            0..=20 => $threshold - 0.05,
            21..=200 => $threshold,
            _ => $threshold + 0.05,
        }
    };
}
