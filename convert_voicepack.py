import torch
import numpy as np
import os
import sys
from pathlib import Path
from huggingface_hub import hf_hub_download

# Fix stdout encoding for Windows
if sys.stdout.encoding != 'utf-8':
    sys.stdout.reconfigure(encoding='utf-8')

VOICES = [
    "diem_trinh", "hung_thinh", "mai_linh", "mai_loan", "manh_dung", 
    "my_yen", "ngoc_huyen", "phat_tai", "thanh_dat", "thuc_trinh", 
    "tuan_ngoc", "storyvert", "duc_an", "duc_duy"
]

def convert_voicepacks(output_dir="voicepacks_npy"):
    os.makedirs(output_dir, exist_ok=True)
    repo_id = "contextboxai/Kokoro-Vietnamese"
    
    for voice in VOICES:
        try:
            print(f"Đang xử lý giọng: {voice}...")
            # Download/resolve from HF Hub
            hf_filename = f"voicepacks/{voice}.pt"
            pt_path = hf_hub_download(repo_id=repo_id, filename=hf_filename)
            
            # Load tensor
            tensor = torch.load(pt_path, map_location='cpu', weights_only=True)
            
            # Chuyển đổi sang numpy array
            if isinstance(tensor, torch.Tensor):
                np_array = tensor.numpy()
            elif isinstance(tensor, list) or isinstance(tensor, tuple):
                np_array = np.stack([t.numpy() for t in tensor])
            else:
                print(f"Định dạng không hỗ trợ cho {voice}: {type(tensor)}")
                continue
                
            out_file = Path(output_dir) / f"{voice}.npy"
            np.save(out_file, np_array)
            print(f"✅ Đã lưu {out_file}")
            
        except Exception as e:
            print(f"❌ Lỗi khi xử lý {voice}: {e}")

if __name__ == "__main__":
    convert_voicepacks()
