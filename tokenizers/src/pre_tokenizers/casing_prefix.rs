use crate::tokenizer::{Decoder, PreTokenizedString, PreTokenizer, Result, SplitDelimiterBehavior};
use crate::tokenizer;
use regex::Regex;
use serde::{Deserialize, Serialize};

const TOKEN_CAPITALISED: &str = "[CAP] ";
const TOKEN_ALL_CAPS: &str = "[ALLCAPS] ";
const TOKEN_MIXED_CASE: &str = "[MIXED] ";

#[derive(Debug, Clone, Serialize)]
pub struct CasingPrefix {
    #[serde(skip)]
    word_regex: Regex,
}

impl PartialEq for CasingPrefix {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<'de> Deserialize<'de> for CasingPrefix {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CasingPrefixVisitor;

        impl<'de> serde::de::Visitor<'de> for CasingPrefixVisitor {
            type Value = CasingPrefix;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct CasingPrefix")
            }

            fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(CasingPrefix::new())
            }
        }

        deserializer.deserialize_unit(CasingPrefixVisitor)
    }
}

impl Default for CasingPrefix {
    fn default() -> Self {
        Self::new()
    }
}

impl CasingPrefix {
    pub fn new() -> Self {
        Self {
            word_regex: Regex::new(r"\w+").unwrap(),
        }
    }

    fn process_word(&self, word: &str) -> String {
        if word.chars().all(|c| c.is_ascii_digit()) {
            word.to_string()
        } else if word.chars().all(|c| c.is_lowercase()) {
            word.to_string()
        } else if word.chars().next().map_or(false, |c| c.is_uppercase()) && word.chars().skip(1).all(|c| c.is_lowercase()) {
            format!("{}{}", TOKEN_CAPITALISED, word.to_lowercase())
        } else if word.chars().all(|c| c.is_uppercase()) {
            format!("{}{}", TOKEN_ALL_CAPS, word.to_lowercase())
        } else {
            format!("{}{}", TOKEN_MIXED_CASE, word.to_lowercase())
        }
    }
}

impl PreTokenizer for CasingPrefix {
    fn pre_tokenize(&self, pretokenized: &mut PreTokenizedString) -> Result<()> {
        pretokenized.split(|_, mut normalized| {
            let mut new_splits = vec![];
            for word in self.word_regex.find_iter(normalized.get()) {
                let processed = self.process_word(word.as_str());
                let mut new_normalized = normalized.slice(tokenizer::normalizer::Range::Original(word.start()..word.end()))
                    .ok_or_else(|| Box::<dyn std::error::Error + Send + Sync>::from("Failed to slice normalized string"))?;
                new_normalized.replace(word.as_str(), &processed)?;
                new_splits.push(new_normalized);
            }
            
            if new_splits.is_empty() {
                Ok(vec![normalized])
            } else {
                Ok(new_splits)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OffsetReferential, OffsetType, PreTokenizer};

    #[test]
    fn test_casing_prefix_pre_tokenizer() {
        let tests = vec![
            (
                "Hello WORLD MixedCase 123 lowercase",
                vec![
                    ("[CAP] hello".to_string(), (0, 5)),
                    ("[ALLCAPS] world".to_string(), (6, 11)),
                    ("[MIXED] mixedcase".to_string(), (12, 21)),
                    ("123".to_string(), (22, 25)),
                    ("lowercase".to_string(), (26, 35)),
                ],
            ),
            (
                "ALL123CAPS 123 mIxEd123CaSe",
                vec![
                    ("[MIXED] all123caps".to_string(), (0, 10)),
                    ("123".to_string(), (11, 14)),
                    ("[MIXED] mixed123case".to_string(), (15, 27)),
                ],
            ),
            // (
            //     "Æsthetic CAFÉ Ångström",
            //     vec![
            //         ("[CAP] æsthetic".to_string(), (0, 8)),
            //         ("[ALLCAPS] café".to_string(), (9, 13)),
            //         ("[CAP] ångström".to_string(), (14, 22)),
            //     ],
            // ),
        ];

        let pretok = CasingPrefix::new();
        for (s, expected) in tests {
            let mut pretokenized = PreTokenizedString::from(s);
            pretok.pre_tokenize(&mut pretokenized).unwrap();
            let result: Vec<_> = pretokenized
                .get_splits(OffsetReferential::Original, OffsetType::Byte)
                .into_iter()
                .map(|(s, o, _)| (s.to_owned(), o))
                .collect();
            assert_eq!(result, expected);
        }
    }
}