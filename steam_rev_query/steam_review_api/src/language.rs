#[cfg(feature = "convenience_structs")]
use serde::{de::Error, Deserialize, Deserializer};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// Languages as represented by the Steam API.
/// Source: https://partner.steamgames.com/doc/store/localization
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Language {
    Arabic,
    Bulgarian,
    SimplifiedChinese,
    TraditionalChinese,
    Czech,
    Danish,
    Dutch,
    English,
    Finnish,
    French,
    German,
    Greek,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Norwegian,
    Polish,
    Portuguese,
    PortugueseBrazilian,
    Romanian,
    Russian,
    SpanishSpain,
    SpanishLatAm,
    Swedish,
    Thai,
    Turkish,
    Ukrainian,
    Vietnamese,
}

#[derive(Debug, Clone, Copy)]
pub struct LangParseError;

impl std::error::Error for LangParseError {}

impl Display for LangParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tried to a parse an unlisted language. \
            Please report! Valve probably added in new languages since I last updated."
        )
    }
}

impl Language {
    /// String representation of how Language appears in queries.
    pub fn as_str(self) -> &'static str {
        use Language::*;
        match self {
            Arabic => "arabic",
            Bulgarian => "bulgarian",
            SimplifiedChinese => "schinese",
            TraditionalChinese => "tchinese",
            Czech => "czech",
            Danish => "danish",
            Dutch => "dutch",
            English => "english",
            Finnish => "finnish",
            French => "french",
            German => "german",
            Greek => "greek",
            Hungarian => "hungarian",
            Italian => "italian",
            Japanese => "japanese",
            Korean => "koreana",
            Norwegian => "norwegian",
            Polish => "polish",
            Portuguese => "portuguese",
            PortugueseBrazilian => "brazilian",
            Romanian => "romanian",
            Russian => "russian",
            SpanishSpain => "spanish",
            SpanishLatAm => "latam",
            Swedish => "swedish",
            Thai => "thai",
            Turkish => "turkish",
            Ukrainian => "ukrainian",
            Vietnamese => "vietnamese",
        }
    }

    /// Shorthand language code as represented by the Steam web API.
    pub fn language_code(self) -> &'static str {
        use Language::*;
        match self {
            Arabic => "ar",
            Bulgarian => "bg",
            SimplifiedChinese => "zh-CN",
            TraditionalChinese => "zh-TW",
            Czech => "cs",
            Danish => "da",
            Dutch => "nl",
            English => "en",
            Finnish => "fi",
            French => "fr",
            German => "de",
            Greek => "el el",
            Hungarian => "hu",
            Italian => "it",
            Japanese => "ja",
            Korean => "ko",
            Norwegian => "no",
            Polish => "pl",
            Portuguese => "pt",
            PortugueseBrazilian => "pt-BR",
            Romanian => "ro",
            Russian => "ru",
            SpanishSpain => "es",
            SpanishLatAm => "es-419",
            Swedish => "sv",
            Thai => "th",
            Turkish => "tr",
            Ukrainian => "uk",
            Vietnamese => "vn",
        }
    }

    /// Language's native name.
    pub fn native_name(self) -> &'static str {
        use Language::*;
        match self {
            Arabic => "العربية",
            Bulgarian => "български език",
            SimplifiedChinese => "简体中文",
            TraditionalChinese => "繁體中文",
            Czech => "čeština",
            Danish => "Dansk",
            Dutch => "Nederlands",
            English => "English",
            Finnish => "Suomi",
            French => "Français",
            German => "Deutsch",
            Greek => "Ελληνικά",
            Hungarian => "Magyar",
            Italian => "Italiano",
            Japanese => "日本語",
            Korean => "한국어",
            Norwegian => "Norsk",
            Polish => "Polski",
            Portuguese => "Português",
            PortugueseBrazilian => "Português-Brasil",
            Romanian => "Română",
            Russian => "Русский",
            SpanishSpain => "Español-España",
            SpanishLatAm => "Español-Latinoamérica",
            Swedish => "Svenska",
            Thai => "ไทย",
            Turkish => "Türkçe",
            Ukrainian => "Українська",
            Vietnamese => "Tiếng Việt",
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Language {
    type Err = LangParseError;

    /// String to Language.
    ///
    /// ## Errors
    /// Returns [LangParseError] if an unsupported language is passed in.
    /// In other words, this function shouldn't fail until Valve adds in new
    /// languages...in which case you should let me know!
    fn from_str(s: &str) -> Result<Self, LangParseError> {
        use Language::*;
        match s {
            "arabic" => Ok(Arabic),
            "bulgarian" => Ok(Bulgarian),
            "schinese" => Ok(SimplifiedChinese),
            "tchinese" => Ok(TraditionalChinese),
            "czech" => Ok(Czech),
            "danish" => Ok(Danish),
            "dutch" => Ok(Dutch),
            "english" => Ok(English),
            "finnish" => Ok(Finnish),
            "french" => Ok(French),
            "german" => Ok(German),
            "greek" => Ok(Greek),
            "hungarian" => Ok(Hungarian),
            "italian" => Ok(Italian),
            "japanese" => Ok(Japanese),
            "koreana" => Ok(Korean),
            "norwegian" => Ok(Norwegian),
            "polish" => Ok(Polish),
            "portuguese" => Ok(Portuguese),
            "brazilian" => Ok(PortugueseBrazilian),
            "romanian" => Ok(Romanian),
            "russian" => Ok(Russian),
            "spanish" => Ok(SpanishSpain),
            "latam" => Ok(SpanishLatAm),
            "swedish" => Ok(Swedish),
            "thai" => Ok(Thai),
            "turkish" => Ok(Turkish),
            "ukrainian" => Ok(Ukrainian),
            "vietnamese" => Ok(Vietnamese),
            _ => Err(LangParseError),
        }
    }
}

#[cfg(feature = "convenience_structs")]
impl<'de> Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Language, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<Language>().map_err(D::Error::custom)
    }
}

#[cfg(feature = "convenience_structs")]
#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::{
        value::{Error, StringDeserializer},
        IntoDeserializer,
    };

    #[test]
    fn good_parse() {
        let english: StringDeserializer<Error> = "english".to_owned().into_deserializer();
        assert_eq!(Language::deserialize(english), Ok(Language::English));
    }
}
