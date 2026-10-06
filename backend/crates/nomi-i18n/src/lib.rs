//! Nomi's languages and the backend's own words in each of them: chat notices, failure
//! explanations, reminder and Home text. Translations live in `locales/<code>.yml` and are
//! compiled in by `rust-i18n`; a key missing from a language falls back to English.
//!
//! What the crew writes is the model's own text: [`Locale::reply_instruction`] tells it which
//! language to answer in.

use std::borrow::Cow;

rust_i18n::i18n!("locales", fallback = "en");

/// A language Nomi speaks. English is the default for anyone who hasn't picked one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Locale {
    #[default]
    En,
    Id,
}

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::En, Locale::Id];

    /// The BCP 47 code stored in `user_preferences.language` and used by the frontend.
    pub fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Id => "id",
        }
    }

    /// `None` for a code Nomi doesn't speak.
    pub fn from_code(code: &str) -> Option<Locale> {
        Locale::ALL.into_iter().find(|l| l.code().eq_ignore_ascii_case(code.trim()))
    }

    /// A stored code, or English when it's missing or unknown.
    pub fn from_code_or_default(code: Option<&str>) -> Locale {
        code.and_then(Locale::from_code).unwrap_or_default()
    }

    /// The language's name in English, for instructions to the model.
    pub fn english_name(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Id => "Indonesian (Bahasa Indonesia)",
        }
    }

    /// Appended to every agent's system prompt so the crew answers in the person's language.
    pub fn reply_instruction(self) -> String {
        let name = self.english_name();
        let tone = match self {
            Locale::En => "",
            Locale::Id => {
                " In Indonesian, be polite, warm and calm, with a light touch of fun: call them \"kamu\" (\"-mu\") \
                 and yourself \"aku\", write full words (no slang, no abbreviations like \"yg\" or \"gak\"), \
                 say \"maaf\" and \"terima kasih\" where it's natural, and use a gentle \"ya\" or \"yuk\" \
                 now and then, never in every sentence."
            }
        };
        format!(
            "The person's app language is {name}. Write every reply to them in {name}, including headings, \
             lists, plans and questions. If they write to you in a different language, answer in theirs instead.{tone}"
        )
    }

    /// The text for `key` in this language.
    pub fn t(self, key: &str) -> String {
        rust_i18n::t!(key, locale = self.code()).into_owned()
    }

    /// The text for `key` with each `%{name}` replaced by its value.
    pub fn tf(self, key: &str, args: &[(&str, &str)]) -> String {
        let mut text: Cow<str> = rust_i18n::t!(key, locale = self.code());
        for (name, value) in args {
            text = Cow::Owned(text.replace(&format!("%{{{name}}}"), value));
        }
        text.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::Locale;

    #[test]
    fn codes_round_trip_and_unknown_codes_fall_back_to_english() {
        for locale in Locale::ALL {
            assert_eq!(Locale::from_code(locale.code()), Some(locale));
        }
        assert_eq!(Locale::from_code_or_default(Some("ID")), Locale::Id);
        assert_eq!(Locale::from_code_or_default(Some("fr")), Locale::En);
        assert_eq!(Locale::from_code_or_default(None), Locale::En);
    }

    #[test]
    fn every_english_key_has_an_indonesian_translation() {
        let available = rust_i18n::available_locales!();
        for locale in Locale::ALL {
            assert!(available.iter().any(|l| l == locale.code()), "{} isn't compiled in", locale.code());
        }
        let english: serde_yaml_keys::Keys = serde_yaml_keys::load(include_str!("../locales/en.yml"));
        let indonesian: serde_yaml_keys::Keys = serde_yaml_keys::load(include_str!("../locales/id.yml"));
        let missing: Vec<_> = english.difference(&indonesian).collect();
        assert!(missing.is_empty(), "missing in id.yml: {missing:?}");
    }

    #[test]
    fn arguments_are_filled_in() {
        assert_eq!(Locale::En.tf("turn.handed_off", &[("agent", "Money")]), "I've passed this to Money. It'll reply here when it's done.");
        assert_eq!(Locale::Id.tf("turn.handed_off", &[("agent", "Money")]), "Sudah aku teruskan ke Money. Balasannya akan muncul di sini begitu selesai, ya.");
    }

    /// Flattened `a.b.c` keys of a locale file, without pulling a YAML parser into the crate.
    mod serde_yaml_keys {
        use std::collections::BTreeSet;
        pub type Keys = BTreeSet<String>;

        pub fn load(text: &str) -> Keys {
            let mut keys = Keys::new();
            let mut path: Vec<(usize, String)> = Vec::new();
            for line in text.lines() {
                let trimmed = line.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("_version") {
                    continue;
                }
                let indent = line.len() - trimmed.len();
                let Some((name, value)) = trimmed.split_once(':') else { continue };
                while path.last().is_some_and(|(i, _)| *i >= indent) {
                    path.pop();
                }
                if value.trim().is_empty() {
                    path.push((indent, name.to_string()));
                } else {
                    let mut key: Vec<&str> = path.iter().map(|(_, n)| n.as_str()).collect();
                    key.push(name);
                    keys.insert(key.join("."));
                }
            }
            keys
        }
    }
}
