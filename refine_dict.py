import json

with open("kokoro-rs/vi_syllables.json", "r", encoding="utf-8") as f:
    data = json.load(f)

refined = {}
for k, v in data.items():
    # Fix 1: Replace 'y' with 'ɨ' (Kokoro uses ɨ for ư/ươ)
    v = v.replace('y', 'ɨ')
    # Fix 2: Remove glottal stop 'ʔ' which sounds harsh in Kokoro
    v = v.replace('ʔ', '')
    # Fix 3: Remove secondary stress 'ˌ' which might confuse the intonation
    v = v.replace('ˌ', '')
    refined[k] = v

with open("kokoro-rs/vi_syllables_refined.json", "w", encoding="utf-8") as f:
    json.dump(refined, f, ensure_ascii=False, indent=2)

print(f"Refined {len(refined)} syllables!")
