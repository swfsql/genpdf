use crate::{OVS, VS};
use semver;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct InfoTranslation {
    pub language: String,
    pub is_translation: bool,
    pub this_project_url: Option<String>,
    pub fetch_translators: bool,
    pub fetch_reviwers: bool,
    pub fetch_progress: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct InfoPerson {
    pub identifier: Option<String>,
    pub rule: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct InfoResource {
    pub rule: Option<String>,
    pub content: Option<String>,
    pub websites: OVS,
    pub description: Option<String>,
    pub persons: OVS,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct InfoCover {
    pub cover_file: String,
    pub cover_dimensions: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct InfoTarget {
    pub name: String,
    pub size: String,
    pub reader: String,
    pub reset_footer_active: bool,
    pub reset_footer_depth: u8,
    pub clear_page_active: bool,
    pub clear_page_depth: u8,
    pub has_parts: bool,
    pub frontmatter_depth: u8,
    pub toc_depth: u8,
    pub mini_toc_active: bool,
    pub mini_toc_depth: u8,
    pub mini_toc_sec_active: bool,
    pub mini_toc_sec_depth: u8,
    pub covers: Vec<InfoCover>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Info {
    #[serde(with = "crate::consts::semver_serde")]
    pub version: semver::Version,
    pub translation: InfoTranslation,
    pub titles: VS,
    pub discussions: Option<Vec<VS>>,
    pub more_infos: Option<Vec<VS>>,
    pub tags: OVS,
    pub tag_prefix: Option<String>,
    pub persons: Option<Vec<InfoPerson>>,
    pub resources: Option<Vec<InfoResource>>,
    pub targets: Vec<InfoTarget>,
    pub content_files: Vec<VS>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct InfoPerson2 {
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Info2 {
    pub authors: Vec<InfoPerson2>,
    pub translators: Vec<InfoPerson2>,
    pub collaborators: Vec<InfoPerson2>,
    pub thanks: Vec<InfoPerson2>,
    pub reviewers: Vec<InfoPerson2>,
}
