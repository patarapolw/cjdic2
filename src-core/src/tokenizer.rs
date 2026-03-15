use std::{collections::HashSet, fs::File, io::BufReader, path::Path};

use serde::Serialize;

use crate::CJDicError;

#[derive(Debug, Serialize)]
pub struct TokenizeSegment {
    pub surface: String,
    /// ```txt
    /// 0	品詞	Part-of-speech
    /// 1	品詞細分類1	Part-of-speech subcategory 1
    /// 2	品詞細分類2	Part-of-speech subcategory 2
    /// 3	品詞細分類3	Part-of-speech subcategory 3
    /// 4	活用形	Conjugation form
    /// 5	活用型	Conjugation type
    /// 6	原形	Base form
    /// 7	読み	Reading
    /// 8	発音	Pronunciation
    /// ```
    /// @see https://lindera.github.io/lindera/dictionaries/ipadic.html
    pub details: Vec<String>,
}

#[derive(Clone)]
pub struct Tokenizer {
    ja_tokenizer: vibrato_rkyv::Tokenizer,
    ja_exclude_pos: HashSet<String>,
}

impl Tokenizer {
    pub fn new(vibrato_dic: impl AsRef<Path>) -> Result<Self, CJDicError> {
        let reader = BufReader::new(File::open(vibrato_dic)?);
        let dictionary = vibrato_rkyv::Dictionary::read(reader)?;
        Ok(Self {
            ja_tokenizer: vibrato_rkyv::Tokenizer::new(dictionary),
            ja_exclude_pos: HashSet::from(
                ["助詞", "助動詞", "記号", "接続詞"].map(|p| p.to_string()),
            ),
        })
    }

    pub fn tokenize(&self, text: String) -> Vec<TokenizeSegment> {
        self.ja_tokenize(text)
    }

    fn ja_tokenize(&self, text: String) -> Vec<TokenizeSegment> {
        let mut worker = self.ja_tokenizer.new_worker();
        worker.reset_sentence(text);
        worker.tokenize();

        let mut output: Vec<TokenizeSegment> = vec![];

        for token in worker.token_iter() {
            let surface = token.surface().to_string();
            let details = token.feature().split(",").map(|s| s.to_string()).collect();

            output.push(TokenizeSegment { surface, details });
        }

        output
    }

    pub fn ja_normalize(&self, text: String) -> Vec<String> {
        let mut segmented: Vec<String> = vec![];
        for t in self.ja_tokenize(text) {
            let mut surface = t.surface;
            let mut pos = None;

            for (i, dt) in t.details.iter().enumerate() {
                match i {
                    0 => pos = Some(dt),
                    6 => {
                        let base = dt.to_string();
                        if base != "*" {
                            surface = base
                        }
                    }
                    _ => {}
                }
            }

            if let Some(p) = pos {
                if self.ja_exclude_pos.contains(p.as_str()) {
                    continue;
                }
            }
            segmented.push(surface);
        }

        segmented
    }
}
