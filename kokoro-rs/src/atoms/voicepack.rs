/****
 * [MODULE]: voicepack
 * Chức năng: Đọc file .npy chứa tensor f32 (Style reference)
 ****/
use ndarray::{Array3, Array2};
use ndarray_npy::read_npy;
use anyhow::Result;

pub struct Voicepack {
    pub styles: Array3<f32>,
}

impl Voicepack {
    pub fn load(npy_path: &str) -> Result<Self> {
        // Voicepack npy có hình dạng [510, 1, 256] 
        let styles: Array3<f32> = read_npy(npy_path)?;
        Ok(Self { styles })
    }

    pub fn get_style_for_length(&self, phoneme_length: usize) -> Result<Array2<f32>> {
        // Lấy style tham chiếu cho độ dài chuỗi phoneme
        let mut idx = if phoneme_length > 0 { phoneme_length - 1 } else { 0 };
        if idx >= self.styles.shape()[0] {
            idx = self.styles.shape()[0] - 1;
        }
        // Lấy mảng [1, 256]
        let style_row = self.styles.slice(ndarray::s![idx..idx+1, 0, ..]).to_owned();
        let reshaped = style_row.into_shape((1, 256))?;
        Ok(reshaped)
    }
}
