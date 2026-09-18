import json

with open("kokoro-rs/vi_syllables.json", "r", encoding="utf-8") as f:
    data = json.load(f)

# Vá lỗi các từ unaccented bị lẫn phát âm tiếng Anh
overrides = {
        "khuyết": "xwˈiɛ↗t",
    "khuyệt": "xwˈiɛʔ↓t",
    "quyết": "kwˈiɛ↗t",
    "quyệt": "kwˈiɛʔ↓t",
    "thuyết": "θwˈiɛ↗t",
    "thuyệt": "θwˈiɛʔ↓t",
    "tuyết": "twˈiɛ↗t",
    "tuyệt": "twˈiɛʔ↓t",
    "duyệt": "zwˈiɛʔ↓t",
    "huyết": "hwˈiɛ↗t",
    "huyệt": "hwˈiɛʔ↓t",
    "truyết": "ʈʂwˈiɛ↗t",
    "truyệt": "ʈʂwˈiɛʔ↓t",
    "nguyệt": "ŋwˈiɛʔ↓t",
    "chuyết": "ʧwˈiɛ↗t",
        "như": "ɲˈɨ",
    "nhưng": "ɲˈɨŋ",
    "là": "lˈaː↘",
    "thì": "θˈi↘",
    "với": "vˈəː↗j",
    "bởi": "bˈəː↓j",
    "chẳng": "ʧˈa↓ŋ",
    "mỗi": "mˈoʔ↗j",
    "khoảng": "xwˈaː↓ŋ",
        "son": "ʂˈɔn",
    "con": "kˈɔn",
    "run": "ɹˈun",
    "man": "mˈan",
    "can": "kˈan",
    "ban": "bˈan",
    "tan": "tˈan",
    "van": "vˈan",
    "pan": "pˈan",
    "men": "mˈen",
    "pen": "pˈen",
    "hen": "hˈen",
    "do": "zˈɔ",
    "to": "tˈɔ",
    "so": "ʂˈɔ",
    "lo": "lˈɔ",
    "no": "nˈɔ",
    "me": "mˈɛ",
    "he": "hˈɛ",
    "be": "bˈɛ",
    "hi": "hˈi",
    "am": "ˈam",
    "in": "ˈin",
    "on": "ˈɔn",
    "an": "ˈan",
    "it": "ˈit"
}
data.update(overrides)

refined = {}
for k, v in data.items():
    # Fix 1: Replace 'y' with 'ɨ' (Kokoro uses ɨ for ư/ươ)
    v = v.replace('y', 'ɨ')
    # Fix 2: Remove glottal stop 'ʔ' which sounds harsh in Kokoro
    # v = v.replace('ʔ', '') # KHÔNG XÓA NỮA! Ký tự này là yếu tố sống còn để tạo ra âm 'Nặng', nếu xóa nó sẽ biến thành dấu Ngã/Hỏi
    # Fix 3: Remove secondary stress 'ˌ' which might confuse the intonation
    v = v.replace('ˌ', '')
    refined[k] = v

with open("kokoro-rs/vi_syllables_refined.json", "w", encoding="utf-8") as f:
    json.dump(refined, f, ensure_ascii=False, indent=2)

print(f"Refined {len(refined)} syllables!")
