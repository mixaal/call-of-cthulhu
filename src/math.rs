// Precompute a lookup table for sine values
const SIN_LUT_SIZE: usize = 3600; // 0.1 degree resolution
lazy_static! {
    static ref SIN_LUT: Vec<f64> = (0..SIN_LUT_SIZE)
        .map(|i| ((i as f64) * std::f64::consts::PI / 1800.0).sin())
        .collect();
}

// Function to get sine value using the lookup table
pub fn sin_lut(angle: f64) -> f64 {
    let normalized_angle = angle.rem_euclid(2.0 * std::f64::consts::PI);
    let index =
        (normalized_angle * (SIN_LUT_SIZE as f64) / (2.0 * std::f64::consts::PI)).round() as usize;
    SIN_LUT[index % SIN_LUT_SIZE]
}

// Precompute a lookup table for sine values using f32
const SIN_LUT_SIZE_F32: usize = 3600; // 0.1 degree resolution
lazy_static! {
    static ref SIN_LUT_F32: Vec<f32> = (0..SIN_LUT_SIZE_F32)
        .map(|i| ((i as f32) * std::f32::consts::PI / 1800.0).sin())
        .collect();
}

// Function to get sine value using the lookup table for f32
pub fn sin_lut_f32(angle: f32) -> f32 {
    let normalized_angle = angle.rem_euclid(2.0 * std::f32::consts::PI);
    let index = (normalized_angle * (SIN_LUT_SIZE_F32 as f32) / (2.0 * std::f32::consts::PI))
        .round() as usize;
    SIN_LUT_F32[index % SIN_LUT_SIZE_F32]
}
