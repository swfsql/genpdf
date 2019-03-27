use crate::{consts, info};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Lang {
    pub from_active: bool,
    pub to_active: bool,
    pub to_dir_name: String,           // pt-BR
    pub set_lang: String,              // brazil (xelatex)
    pub title: String,                 // Portuguese (Brazilian)
    pub from_url: Option<String>,      // https://crowdin.com/project/ancap-ch/
    pub from_dir_name: Option<String>, // from_en
}

#[derive(Debug)]
pub struct Defaults {
    pub info: info::Info,
    pub info2: info::Info2,
    pub target: String,
    pub info_target: info::InfoTarget,

    pub sent_initial: String,

    pub all_langs: Vec<Lang>,
    pub def_lang: Lang,
    pub other_langs: Vec<Lang>,

    pub consts: consts::Consts,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DefaultsFile {
    pub info: info::Info,
    pub info2: info::Info2,
    pub target: String,
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
    pub base_dir: String,
    pub from_dir: String,
    pub lang_dir: String,
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
