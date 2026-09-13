/****
 * [MODULE]: inference
 * Chức năng: Tải mô hình ONNX và xử lý suy luận (Hỗ trợ ort 2.0.0-rc)
 ****/
use ort::{
    init, 
    session::Session, 
    session::builder::GraphOptimizationLevel, 
    value::Tensor, 
    inputs
};
use ndarray::{Array1, Array2};
use std::sync::Arc;
use anyhow::Result;

pub struct KokoroEngine {
    session: Session,
}

impl KokoroEngine {
    pub fn new(model_path: &str) -> Result<Self> {
        // Khởi tạo ORT Environment một lần duy nhất
        let _ = init().with_name("kokoro_env").commit();

        let session = Session::builder()
            .map_err(|e| anyhow::anyhow!("Builder error: {:?}", e))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("Opt error: {:?}", e))?
            .with_intra_threads(1)
            .map_err(|e| anyhow::anyhow!("Thread error: {:?}", e))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow::anyhow!("Commit error: {:?}", e))?;

        Ok(Self {
            session,
        })
    }

    pub fn synthesize(
        &mut self, 
        input_ids: Vec<i64>, 
        ref_s: Array2<f32>, 
        speed: f32
    ) -> Result<Vec<f32>> {
        let seq_len = input_ids.len();
        
        let input_ids_tensor = Tensor::from_array((vec![1, seq_len], input_ids))
            .map_err(|e| anyhow::anyhow!("Tensor error: {:?}", e))?;
        
        // Chuyển ref_s sang vec phẳng
        let ref_s_vec: Vec<f32> = ref_s.iter().cloned().collect();
        let ref_s_tensor = Tensor::from_array((vec![1, 256], ref_s_vec))
            .map_err(|e| anyhow::anyhow!("Tensor error: {:?}", e))?;
        
        let speed_tensor = Tensor::from_array((vec![1], vec![speed]))
            .map_err(|e| anyhow::anyhow!("Tensor error: {:?}", e))?;
        
        // Chạy session
        let outputs = self.session.run(inputs![
            input_ids_tensor,
            ref_s_tensor,
            speed_tensor
        ]).map_err(|e| anyhow::anyhow!("Run error: {:?}", e))?;
        
        // Output audio
        let (_shape, slice) = outputs[0].try_extract_tensor::<f32>()
            .map_err(|e| anyhow::anyhow!("Extract error: {:?}", e))?;
        
        Ok(slice.to_vec())
    }
}
