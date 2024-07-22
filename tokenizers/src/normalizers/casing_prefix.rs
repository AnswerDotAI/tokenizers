use crate::tokenizer::{NormalizedString, Normalizer, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

const TOKEN_CAPITALISED: &str = "[CAP]";
const TOKEN_ALL_CAPS: &str = "[ALLCAPS]";
const TOKEN_MIXED_CASE: &str = "[MIXED]";

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
            // Return digits-only content as is
            word.to_string()
        } else if word.chars().all(|c| c.is_lowercase()) {
            // Return lowercase words as is
            word.to_string()
        } else if word.chars().next().map_or(false, |c| c.is_uppercase()) && word[1..].chars().all(|c| c.is_lowercase()) {
            format!("{}{}", TOKEN_CAPITALISED, word.to_lowercase())
        } else if word.chars().all(|c| c.is_uppercase()) {
            format!("{}{}", TOKEN_ALL_CAPS, word.to_lowercase())
        } else {
            format!("{}{}", TOKEN_MIXED_CASE, word.to_lowercase())
        }
    }
}

impl Normalizer for CasingPrefix {
    fn normalize(&self, normalized: &mut NormalizedString) -> Result<()> {
        let text = normalized.get().to_string();
        let processed_text: String = self
            .word_regex
            .find_iter(&text)
            .map(|m| self.process_word(m.as_str()))
            .collect::<Vec<String>>()
            .join(" ");

        normalized.replace(&text, &processed_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casing_prefix_normalizer() {
        let original = "Hello WORLD MixedCase 123 lowercase";
        let expected = "[CAP]hello [ALLCAPS]world [MIXED]mixedcase 123 lowercase";

        let mut n = NormalizedString::from(original);
        CasingPrefix::new().normalize(&mut n).unwrap();

        assert_eq!(n.get(), expected);
    }

    #[test]
    fn test_casing_prefix_edge_cases() {
        let original = "ALL123CAPS 123 mIxEd123CaSe";
        let expected = "[MIXED]all123caps 123 [MIXED]mixed123case";

        let mut n = NormalizedString::from(original);
        CasingPrefix::new().normalize(&mut n).unwrap();

        assert_eq!(n.get(), expected);
    }
}