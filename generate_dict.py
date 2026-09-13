import json
import sys
import time

try:
    from vig2p import phonemize_text
except ImportError:
    print("vig2p not installed")
    sys.exit(1)

initials = ["", "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh", "p", "ph", "q", "r", "s", "t", "th", "tr", "v", "x"]
vowels = ["a", "á", "à", "ả", "ã", "ạ", "ă", "ắ", "ằ", "ẳ", "ẵ", "ặ", "â", "ấ", "ầ", "ẩ", "ẫ", "ậ", "e", "é", "è", "ẻ", "ẽ", "ẹ", "ê", "ế", "ề", "ể", "ễ", "ệ", "i", "í", "ì", "ỉ", "ĩ", "ị", "o", "ó", "ò", "ỏ", "õ", "ọ", "ô", "ố", "ồ", "ổ", "ỗ", "ộ", "ơ", "ớ", "ờ", "ở", "ỡ", "ợ", "u", "ú", "ù", "ủ", "ũ", "ụ", "ư", "ứ", "ừ", "ử", "ữ", "ự", "y", "ý", "ỳ", "ỷ", "ỹ", "ỵ", "oa", "oá", "oà", "oả", "oã", "oạ", "oe", "oé", "oè", "oẻ", "oẽ", "oẹ", "uê", "uế", "uề", "uể", "uễ", "uệ", "uy", "uý", "uỳ", "uỷ", "uỹ", "uỵ", "ua", "uá", "uà", "uả", "uã", "uạ", "ưa", "ứa", "ừa", "ửa", "ữa", "ựa", "ia", "ía", "ìa", "ỉa", "ĩa", "ịa", "ya", "ýa", "ỳa", "ỷa", "ỹa", "ỵa", "iê", "iế", "iề", "iể", "iễ", "iệ", "yê", "yế", "yề", "yể", "yễ", "yệ", "ươ", "ướ", "ườ", "ưở", "ưỡ", "ượ", "uô", "uố", "uồ", "uổ", "uỗ", "uộ", "oai", "oái", "oài", "oải", "oãi", "oại", "oao", "oáo", "oào", "oảo", "oão", "oạo", "oay", "oáy", "oày", "oảy", "oãy", "oạy", "oeo", "oéo", "oèo", "oẻo", "oẽo", "oẹo", "uân", "uấn", "uần", "uẩn", "uẫn", "uận", "uất", "uật", "uyên", "uyến", "uyền", "uyển", "uyễ", "uyện"]
finals = ["", "c", "ch", "m", "n", "ng", "nh", "p", "t", "i", "y", "o", "u"]

def build():
    print("Generating combinations...")
    syllables = set()
    for i in initials:
        for v in vowels:
            for f in finals:
                word = i + v + f
                syllables.add(word)
    
    print(f"Total combinations: {len(syllables)}")
    
    res = {}
    words = list(syllables)
    
    # Process in batches or individually
    print("Processing G2P...")
    start = time.time()
    
    # Just do a batch
    # Actually vig2p supports processing many words, but phonemize_text is per string
    # Let's just loop
    for idx, w in enumerate(words):
        if idx % 5000 == 0:
            print(f"Processed {idx}/{len(words)}")
        try:
            phon = phonemize_text(w)
            # Only save if it doesn't just return the word itself (which means it couldn't process it completely)
            # Or just save everything.
            res[w] = phon
        except Exception:
            pass
            
    print(f"Done in {time.time() - start:.2f}s")
    
    with open("kokoro-rs/vi_syllables.json", "w", encoding="utf-8") as f:
        json.dump(res, f, ensure_ascii=False, indent=2)
    print("Saved to kokoro-rs/vi_syllables.json")

if __name__ == "__main__":
    build()
