/****
 * [MODULE]: inference
 * Chức năng: Tải mô hình ONNX và xử lý suy luận (Hỗ trợ ort 2.0.0-rc)
 ****/
use ort::{init, Session, Tensor, GraphOptimizationLevel, inputs};
use ndarray::{Array1, Array2};
use std::sync::Arc;
use anyhow::Result;

pub struct KokoroEngine {
    session: Arc<Session>,
}

impl KokoroEngine {
    pub fn new(model_path: &str) -> Result<Self> {
        // Khởi tạo ORT Environment một lần duy nhất
        let _ = init().with_name("kokoro_env").commit();

        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)?
            .commit_from_file(model_path)?;

        Ok(Self {
            session: Arc::new(session),
        })
    }

    pub fn synthesize(
        &self, 
        input_ids: Vec<i64>, 
        ref_s: Array2<f32>, 
        speed: f32
    ) -> Result<Vec<f32>> {
        let seq_len = input_ids.len();
        let input_ids_array = Array2::from_shape_vec((1, seq_len), input_ids)?;
        
        let input_ids_tensor = Tensor::from_array(input_ids_array)?;
        let ref_s_tensor = Tensor::from_array(ref_s)?;
        
        let speed_array = Array1::from_vec(vec![speed]);
        let speed_tensor = Tensor::from_array(speed_array)?;
        
        // Chạy session
        let outputs = self.session.run(inputs![
            input_ids_tensor,
            ref_s_tensor,
            speed_tensor
        ]?)?;
        
        // Output audio
        let audio_tensor = outputs[0].try_extract_tensor::<f32>()?;
        let audio_data = audio_tensor.view().iter().cloned().collect();
        
        Ok(audio_data)
    }
}
