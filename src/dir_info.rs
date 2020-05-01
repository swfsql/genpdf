use crate::{consts, info};
// use language_tags::LanguageTag;
use std::path::{Path, PathBuf};

pub use language_tags::LanguageTag;

pub fn tag_try_into_whatlang(tag: LanguageTag) -> Result<whatlang::Lang, failure::Error> {
    match tag.to_string().as_ref() {
        "en" => Ok(whatlang::Lang::Eng),
        "pt-BR" => Ok(whatlang::Lang::Por),
        "th" => Ok(whatlang::Lang::Tha),
        other => Err(feh!("{:?}", other)),
    }
}

pub mod langtag_serde {
    use super::LanguageTag;

    pub fn serialize<S>(lang: &LanguageTag, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&lang.to_string())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<LanguageTag, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Deserialize;
        use std::str::FromStr;
        let string: String = String::deserialize(d)?;
        LanguageTag::from_str(&string).map_err(serde::de::Error::custom)
    }
}

pub mod langtagvec_serde {
    use super::LanguageTag;

    // reference:
    // https://serde.rs/impl-serialize.html
    pub fn serialize<S>(langs: &[LanguageTag], s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(langs.len()))?;
        for e in langs {
            seq.serialize_element(&e.to_string())?;
        }
        seq.end()
    }

    // reference:
    // https://github.com/serde-rs/serde/issues/723#issuecomment-382501277
    pub fn deserialize<'de, D>(d: D) -> Result<Vec<LanguageTag>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper(#[serde(with = "super::langtag_serde")] LanguageTag);

        use serde::de::Deserialize;
        let v: Vec<Wrapper> = Vec::deserialize(d)?;
        Ok(v.into_iter().map(|Wrapper(a)| a).collect())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Lang {
    pub from_active: bool,
    pub to_active: bool,
    #[serde(with = "langtag_serde")]
    pub to_dir_name: LanguageTag, // pt-BR
    pub set_lang: String,              // brazil (xelatex)
    pub title: String,                 // Portuguese (Brazilian)
    pub from_url: Option<String>,      // https://crowdin.com/project/ancap-ch/
    pub from_dir_name: Option<String>, // from_en
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Defaults {
    pub info: info::Info,
    pub info2: info::Info2,
    pub target: info::TargetName,
    pub info_target: info::InfoTarget,

    pub sent_initial: String,

    pub all_langs: Vec<Lang>,
    pub def_lang: Lang,
    pub other_langs: Vec<Lang>,

    pub consts: consts::Consts,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct DirInfo {
    /// eg. "/home/thi/ancap.ch/to_dir"
    pub base_dir: String,

    /// eg. "from_en"
    pub from_dir: String,

    /// eg. "tl"
    pub lang_dir: String,

    /// eg. "Universailly Preferable Behaviour"
    pub proj_dir: String,

    pub info: info::Info,
}

impl DirInfo {
    pub fn fulldir(&self) -> PathBuf {
        Path::new(&self.base_dir)
            .join(&self.from_dir)
            .join(&self.lang_dir)
            .join(&self.proj_dir)
    }
    pub fn fulldir_str(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.base_dir, self.from_dir, self.lang_dir, self.proj_dir
        )
    }
}
