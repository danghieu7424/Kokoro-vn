/****
 * [MODULE]: dsp
 * Chức năng: Xử lý tín hiệu số (Digital Signal Processing) thuần Rust.
 * Hỗ trợ Pitch Shifting dựa trên nguyên lý bù trừ tốc độ (Time-scale modification)
 * kết hợp với nội suy Hermite/Cubic.
 ****/

pub fn cubic_interp(y0: f32, y1: f32, y2: f32, y3: f32, mu: f32) -> f32 {
    let a0 = -0.5 * y0 + 1.5 * y1 - 1.5 * y2 + 0.5 * y3;
    let a1 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
    let a2 = -0.5 * y0 + 0.5 * y2;
    let a3 = y1;
    a0 * mu * mu * mu + a1 * mu * mu + a2 * mu + a3
}

/// Pitch shift bằng phương pháp Resampling + bù tốc độ Kokoro.
/// ratio = 2^(cents / 1200)
pub fn resample_audio(input: &[f32], ratio: f32) -> Vec<f32> {
    if input.is_empty() {
        return vec![];
    }
    
    let new_length = (input.len() as f32 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(new_length);
    
    for i in 0..new_length {
        let exact_pos = i as f32 * ratio;
        let idx = exact_pos.floor() as usize;
        let mu = exact_pos - exact_pos.floor();
        
        let y0 = if idx > 0 { input[idx - 1] } else { input[0] };
        let y1 = input[idx];
        let y2 = if idx + 1 < input.len() { input[idx + 1] } else { input[input.len() - 1] };
        let y3 = if idx + 2 < input.len() { input[idx + 2] } else { input[input.len() - 1] };
        
        let sample = cubic_interp(y0, y1, y2, y3, mu);
        output.push(sample);
    }
    
    output
}
