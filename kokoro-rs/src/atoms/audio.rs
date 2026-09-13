/****
 * [MODULE]: audio
 * Chức năng: Lưu âm thanh ra file WAV
 ****/
use hound;
use anyhow::Result;

pub fn save_wav(audio_data: &[f32], filepath: &str, sample_rate: u32) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    
    let mut writer = hound::WavWriter::create(filepath, spec)?;
    for &sample in audio_data {
        writer.write_sample(sample)?;
    }
    
    Ok(())
}
