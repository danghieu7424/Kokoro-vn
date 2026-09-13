import soundfile as sf
from kokoro_vietnamese import KokoroVietnamese
import sys
import os

# Khắc phục lỗi in tiếng Việt trên Terminal Windows
if sys.stdout.encoding != 'utf-8':
    sys.stdout.reconfigure(encoding='utf-8')

def main():
    voices = [
        "diem_trinh"
    ]
    # voices = [
    #     "diem_trinh", "hung_thinh", "mai_linh", "manh_dung", 
    #     "my_yen", "ngoc_huyen", "phat_tai", "thanh_dat", "thuc_trinh", 
    #     "tuan_ngoc", "duc_an", "duc_duy"
    # ]
    
    text = "Tôi là người việt nam"
    print("Bắt đầu thử nghiệm tạo âm thanh với tất cả các giọng...")
    
    # Tạo thư mục lưu kết quả nếu chưa có
    output_dir = "outputs/all_voices"
    os.makedirs(output_dir, exist_ok=True)
    
    for voice in voices:
        print(f"\n--- Đang xử lý giọng: {voice} ---")
        try:
            # Khởi tạo TTS cho từng giọng
            tts = KokoroVietnamese(device="cpu", voice=voice)
            
            # Tạo âm thanh
            audio, phonemes = tts.synthesize(text)
            
            # Lưu thành file wav
            output_path = os.path.join(output_dir, f"{voice}.wav")
            sf.write(output_path, audio, 24000)
            print(f"✅ Đã lưu thành công: {output_path}")
        except Exception as e:
            print(f"❌ Lỗi khi xử lý giọng {voice}: {e}")
            
    print(f"\n🎉 Hoàn tất! Tất cả các file đã được lưu vào thư mục: {output_dir}")

if __name__ == "__main__":
    main()
