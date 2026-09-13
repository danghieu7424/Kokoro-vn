/****
 * [MODULE]: g2p
 * Chức năng: PoC Tokenizer - Ánh xạ Phoneme thành Token IDs dựa vào config.json
 ****/
use std::collections::HashMap;
use std::fs;
use serde::Deserialize;
use anyhow::Result;

#[derive(Deserialize)]
struct Config {
    vocab: HashMap<String, i64>,
}

pub struct Tokenizer {
    vocab: HashMap<char, i64>,
}

impl Tokenizer {
    pub fn new(config_path: &str) -> Result<Self> {
        let config_str = fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;
        
        let mut vocab = HashMap::new();
        for (k, v) in config.vocab {
            if k.chars().count() == 1 {
                vocab.insert(k.chars().next().unwrap(), v);
            }
        }
        
        // Add default tokens
        vocab.insert('0', 0); // padding
        
        Ok(Self { vocab })
    }

    pub fn encode(&self, phonemes: &str) -> Vec<i64> {
        let mut ids = vec![0]; // START token (usually 0)
        for c in phonemes.chars() {
            if let Some(&id) = self.vocab.get(&c) {
                ids.push(id);
            }
        }
        ids.push(0); // END token
        ids
    }
}
