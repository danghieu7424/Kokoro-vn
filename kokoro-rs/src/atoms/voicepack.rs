/****
 * [MODULE]: voicepack
 * Chức năng: Đọc file .npy chứa tensor f32 (Style reference)
 ****/
use ndarray::{Array2, ArrayView2};
use ndarray_npy::read_npy;
use anyhow::Result;

pub struct Voicepack {
    pub styles: Array2<f32>,
}

impl Voicepack {
    pub fn load(npy_path: &str) -> Result<Self> {
        // Voicepack npy có hình dạng [N, 256] 
        // N là số lượng độ dài (từ 1 đến 510)
        let styles: Array2<f32> = read_npy(npy_path)?;
        Ok(Self { styles })
    }

    pub fn get_style_for_length(&self, phoneme_length: usize) -> Result<ArrayView2<f32>> {
        // Lấy style tham chiếu cho độ dài chuỗi phoneme
        // Trong python: ref_s = self.voicepack[len(ps) - 1]
        let idx = if phoneme_length > 0 { phoneme_length - 1 } else { 0 };
        // Lấy một hàng và định dạng lại thành [1, 256]
        let style_row = self.styles.slice(ndarray::s![idx..idx+1, ..]);
        Ok(style_row)
    }
}
