import os
import torch
import numpy as np
from huggingface_hub import hf_hub_download

voices = [
    "diem_trinh", "hung_thinh", "mai_linh", "manh_dung", 
    "my_yen", "ngoc_huyen", "phat_tai", "thanh_dat", "thuc_trinh", 
    "tuan_ngoc", "duc_an", "duc_duy"
]

repo_id = "contextboxai/Kokoro-Vietnamese"
output_dir = "voicepacks_npy"
os.makedirs(output_dir, exist_ok=True)

print("Downloading and converting voices to .npy...")

for voice in voices:
    filename = f"voicepacks/{voice}.pt"
    print(f"Processing {voice}...")
    try:
        # Download .pt
        local_path = hf_hub_download(repo_id=repo_id, filename=filename)
        
        # Load with torch
        tensor = torch.load(local_path, map_location="cpu", weights_only=True)
        
        # Convert to numpy
        arr = tensor.numpy()
        
        # Save to .npy
        npy_path = os.path.join(output_dir, f"{voice}.npy")
        np.save(npy_path, arr)
        print(f"  -> Saved {npy_path}")
    except Exception as e:
        print(f"  -> Failed to process {voice}: {e}")

print("All done!")
