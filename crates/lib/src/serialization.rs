use serde::Serializer;

/// Serialize f32 with proper rounding (2 decimal places for small numbers)
/// Outputs integer format for whole numbers (1 instead of 1.0)
pub fn serialize_f32<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert to f64 for better precision during rounding
    let v = *value as f64;

    // Round to 2 decimal places (handles 0.05, 0.9 etc)
    let rounded = (v * 100.0).round() / 100.0;

    // If it's a whole number, serialize as integer
    if rounded.fract() == 0.0 {
        serializer.serialize_i64(rounded as i64)
    } else {
        // Serialize as f64 to get cleaner output
        serializer.serialize_f64(rounded)
    }
}

/// Serialize f64 with proper rounding
/// Outputs integer format for whole numbers
pub fn serialize_f64<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Round to 2 decimal places
    let rounded = (*value * 100.0).round() / 100.0;

    // If it's a whole number, serialize as integer
    if rounded.fract() == 0.0 {
        serializer.serialize_i64(rounded as i64)
    } else {
        serializer.serialize_f64(rounded)
    }
}
