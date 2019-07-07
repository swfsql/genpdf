use crate::dir_info;
use crate::{OVS, VS};
use dir_info::LanguageTag;
use semver;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct InfoTranslation {
    // TODO: replace by enum
    /// eg. "tl"
    #[serde(with = "dir_info::langtag_serde")]
    pub language: LanguageTag,

    #[serde(with = "dir_info::langtagvec_serde")]
    pub other_languages: Vec<LanguageTag>,

    pub is_translation: bool,

    // TODO: replace by option<Url>
    pub this_project_url: Option<String>,

    // TODO: replace by Option<Url>
    pub fetch_translators: bool,

    // TODO: replace by Option<Url>
    pub fetch_reviwers: bool,

    // TODO: replace by Option<Url>
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
    // TODO: replace by path
    /// eg. "Pasture_and_Rail_Fence.jpg"
    pub cover_file: String,

    pub cover_dimensions: Vec<u32>,
}

#[derive(Serialize, Deserialize, Display, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum TargetName {
    // more clearpages
    Book,
    // less clearpages
    Article,
}

#[derive(Serialize, Deserialize, Display, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum TargetSize {
    A0paper,
    A1paper,
    A2paper,
    A3paper,
    A4paper,
    A5paper,
    A6paper,
    B0paper,
    B1paper,
    B2paper,
    B3paper,
    B4paper,
    B5paper,
    B6paper,
    C0paper,
    C1paper,
    C2paper,
    C3paper,
    C4paper,
    C5paper,
    C6paper,
    B0j,
    B1j,
    B2j,
    B3j,
    B4j,
    B5j,
    BBj,
    Ansiapaper,
    Ansibpaper,
    Ansicpaper,
    Ansidpaper,
    Ansiepaper,
    Letterpaper,
    Executivepaper,
    Legalpaper,
    // HD720 are into A4paper area
    // = 7.3774614536439in x 13.1154870287in (portrait)
    // = 1280 x 720 = 16 x 9 (landscape)
    Hd720,
    // WXGA+ area into A4 paper area
    // = 7.7765271812035in x 12.442443489926in (portrait)
    // = 1440 x 1050 = 16 x 10 (landscape)
    Wxgap,
    // XGA area into A4 paper area
    // = 8.5151174390022in x 11.35348991867in (portrait)
    // = 1024 x 768 = 4 x 3 (landscape)
    Xga,
}

#[derive(Serialize, Deserialize, Display, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum TargetOrientation {
    Portrait,  // normal print
    Landscape, // widescreen-like
}

#[derive(Serialize, Deserialize, Display, Debug, Clone, PartialEq, Eq, Hash)]
#[strum(serialize_all = "camel_case")]
pub enum TargetFontSize {
    #[serde(rename = "8pt")]
    _8pt,
    #[serde(rename = "9pt")]
    _9pt,
    #[serde(rename = "10pt")]
    _10pt,
    #[serde(rename = "11pt")]
    _11pt,
    #[serde(rename = "12pt")]
    _12pt,
    #[serde(rename = "14pt")]
    _14pt,
    #[serde(rename = "17pt")]
    _17pt,
    #[serde(rename = "20pt")]
    _20pt,
    #[serde(rename = "25pt")]
    _25pt,
    #[serde(rename = "30pt")]
    _30pt,
    #[serde(rename = "36pt")]
    _36pt,
}

#[derive(Serialize, Deserialize, Display, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum TargetReader {
    // twoside, openright
    Print,
    // oneside, openany
    Digital,
}

#[derive(Serialize, Deserialize, Display, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum TargetEngine {
    // powerful microtype
    Latex,
    // good multi-lang
    Xelatex,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct InfoTarget {
    pub name: TargetName,
    pub size: TargetSize,
    pub orientation: TargetOrientation,
    pub font_size: TargetFontSize,
    pub reader: TargetReader,
    pub engine: TargetEngine,
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

    // TODO: replace by structure
    pub discussions: Option<Vec<VS>>,

    // TODO: replace by structure
    pub more_infos: Option<Vec<VS>>,

    // TODO: replace by structure
    pub tags: OVS,

    // TODO: replace by structure
    pub tag_prefix: Option<String>,

    pub persons: Option<Vec<InfoPerson>>,

    pub resources: Option<Vec<InfoResource>>,

    pub targets: Vec<InfoTarget>,

    // TODO: replace by paths
    /// eg. [["01_pref2nd_ed.md", ""], ["02_pref1st_ed.md", ""], ..]
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
