#![allow(clippy::trivial_regex)]

use crate::dir_info;
use crate::VS;
use regex::Regex;
use semver;
use std::convert::TryFrom;
use std::path::Path;
use tera::Tera;
// use serde as serde_lib;
use dir_info::LanguageTag;

// from_dir is used as:
// - $GEN/from_dir/
// which contains original text, such as in $GEN/from_dir/from_en/
//
// to_dir is used as:
// - $GEN/to_dir/
// and it refers to the translation work
//
// the translated work are classified by their "original" language
// - $GEN/to_dir/from_en/pt-BR
// would contain all texts translated into portuguese that came from english
//
// TODO: don't bother with the "original" language.
// because if a text from lang A is translated to B and it's C lang translation
// began, C may consider both A and B as "originals".
// yet, even if a crew is translating using both A and B as basis,
// only one C work files should exist
// therefore, the C work files must not be classified by "original" lang
//
//
//
//
//
//
//

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Consts {
    #[serde(with = "semver_serde")]
    pub min_ver: semver::Version,
    pub passages: u8,
    pub cover_nodes: Vec<String>,
    pub all_langs_from_dir: String,
    pub all_langs_to_dir: String,
    pub output_dir: String,
    pub initials: Vec<VS>,
    pub num_cpu: u8,
    pub all_langs: Vec<dir_info::Lang>,
}

use std::collections::HashSet;
impl Consts {
    pub fn get_active_langs(&self) -> HashSet<LanguageTag> {
        self.all_langs.iter().fold(HashSet::new(), |mut hs, l| {
            if l.to_active {
                hs.insert(l.to_dir_name.clone());
                hs
            } else {
                hs
            }
        })
    }
}

pub mod semver_serde {
    pub fn serialize<S>(version: &semver::Version, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<semver::Version, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Deserialize;
        let string: String = String::deserialize(d)?;
        semver::Version::parse(&string).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&Path> for Consts {
    type Error = failure::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        use std::fs;
        let tomlc: String = fs::read_to_string(path)?;
        Ok(toml::from_str(tomlc.as_str())?)
    }
}

impl Into<Vec<dir_info::DirInfo>> for &Consts {
    fn into(self) -> Vec<dir_info::DirInfo> {
        let consts = self;
        // for each language
        consts
            .all_langs
            .iter()
            // ignore deactivated ones
            .filter(|lang| lang.from_active)
            .filter_map(|lang| {
                // eg. "from_en"
                let from_dir_name = lang.from_dir_name.clone()?;

                // eg. "en"
                dbg!("Reading towards language directory:");
                println!("{:?}", &lang.to_dir_name);

                // eg "$ANCAP/to_dir/from_en"
                let dir = format!("{}/{}", &consts.all_langs_to_dir, &from_dir_name);

                use std::fs;

                let dir = fs::read_dir(&dir) //
                    .unwrap_or_else(|_e| {
                        panic!(
                            "{}",
                            &fh!("Failed to open the contents of {} directory.", &dir)
                        )
                    });

                let oks: Vec<dir_info::DirInfo> = dir
                    .filter_map(|lang_dir| {
                        // dbg!("{:?}", lang_dir);

                        let lang_dir: std::fs::DirEntry = lang_dir.unwrap_or_else(|_e| {
                            panic!("{}", &fh!("Failed to open language directory"))
                        });

                        // only accept directories
                        if lang_dir
                            .metadata()
                            .unwrap_or_else(|_e| panic!("{}", &fh!()))
                            .is_file()
                        {
                            return None;
                        }

                        // eg. "$ANCAP/to_dir/from_en/fa"
                        let lang_dir = lang_dir.path();

                        // eg. "fa"
                        let lang_dir_name = lang_dir
                            .file_name()
                            .unwrap_or_else(|| panic!("{}", &fh!()))
                            .to_string_lossy()
                            .to_string();

                        // ignore some specific dirs
                        if lang_dir_name == ".git" || lang_dir_name == "_asset" {
                            return None;
                        }

                        use std::str::FromStr;
                        let lang_dir_name = LanguageTag::from_str(&lang_dir_name)
                            .unwrap_or_else(|_e| panic!("{}", &fh!("{:?}", &lang_dir_name)));
                        // .map_err(wfeh!())?;
                        let lang_dir_name: LanguageTag = lang_dir_name;

                        // filter out if such to_lang is deactivated
                        if !consts.get_active_langs().contains(&lang_dir_name) {
                            return None;
                        }

                        let proj_dirs = fs::read_dir(lang_dir) //
                            .unwrap_or_else(|_| panic!("{}", &fh!()));
                        let dir_infos: Vec<dir_info::DirInfo> = proj_dirs
                            .filter_map(|proj_dir| {
                                // eg. "$ANCAP/to_dir/from_en/ms/UPB"
                                let proj_dir =
                                    proj_dir.unwrap_or_else(|_| panic!("{}", &fh!())).path();

                                // eg. "UPB"
                                let proj_dir_name = proj_dir
                                    .file_name()
                                    .unwrap_or_else(|| panic!("{}", &fh!()))
                                    .to_string_lossy()
                                    .to_string();

                                /*
                                // previous cmds, for yml files
                                let yml = match File::open(proj_dir.join("info.yml")) {
                                    Ok(yml) => Some(yml),
                                    Err(e) => None,
                                }?;
                                let info = serde_yaml::from_reader(yml)
                                    .map_err(|e| format!("Failed to parse the yml info file contents in folder {}. Error: {}", proj_dir_name, e));
                                */

                                // TODO: panic on errors
                                // eg. "version = "0.1.4"\ntitles = ..."
                                let tomlc = fs::read_to_string(proj_dir.join("info.toml")).ok()?;

                                let info: crate::info::Info = toml::from_str(&tomlc)
                                    .unwrap_or_else(|_| {
                                        panic!(
                                            "{}",
                                            &fh!(
                                    "Failed to parse the toml info file contents <{:?}>",
                                    &proj_dir.join("info.toml")
                                )
                                        )
                                    });

                                if info.version < consts.min_ver {
                                    return None;
                                }

                                /*
                                // previous, for toml file creation (from yml files)
                                {
                                    let mut toml = File::create(proj_dir.join("info.toml"))
                                        .expect(&fh!("failed for ({}) {}", lang_dir_name, proj_dir_name));

                                    use toml::Value;
                                    let toml_value: Value = Value::try_from(&info).unwrap();

                                    let toml_str: String = toml::to_string(&toml_value).unwrap();


                                    toml.write_all(toml_str.as_bytes()).unwrap();
                                    dbg!("created one for {}", &lang_dir_name);
                                }
                                */

                                let dir_info = dir_info::DirInfo {
                                    base_dir: consts.all_langs_to_dir.clone(),
                                    from_dir: from_dir_name.clone(),
                                    lang_dir: lang_dir_name.to_string(),
                                    proj_dir: proj_dir_name,
                                    info,
                                };

                                // dbg!("\ninfo:\n{:?}\n", &dir_info.info);
                                dbg!("read info for a project in:");
                                println!("{}", &lang_dir_name.to_string());

                                Some(dir_info)
                            })
                            .collect::<Vec<dir_info::DirInfo>>();

                        Some(dir_infos)
                    })
                    .fold(vec![], |mut vs, v| {
                        vs.extend(v);
                        vs
                    });
                dbg!("different projects were identified: ");
                dbg!(&oks.len());
                Some(oks)
            })
            .fold(vec![], |mut vs, v| {
                vs.extend(v);
                vs
            })
    }
}

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = compile_templates!("templates/**/*");
        tera.autoescape_on(vec![".tex"]);
        //tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
    pub static ref RE_FORWARD_ARROW: Regex =
        Regex::new("\\{->|\\{-&gt;").unwrap(); // some macros may output -> as {-&gt;

    pub static ref RE_SUB_HASH_SPACE_HASH: Regex = Regex::new("# #").unwrap();
    pub static ref RE_SUB_HASH_DOWNGRADE: Regex = Regex::new("(?m)^#(#*)([^#]*)$").unwrap();

    pub static ref RE_SYMB_AMPER: Regex = Regex::new("&").unwrap();
    pub static ref RE_SYMB_DOLLAR: Regex = Regex::new("\\$").unwrap();
    pub static ref RE_SYMB_CURLY_BRACK: Regex = Regex::new("\\{").unwrap();
    pub static ref RE_SYMB_CURLY_BRACK2: Regex = Regex::new("\\}").unwrap();
    pub static ref RE_SYMB_PERCENT: Regex = Regex::new("%").unwrap();
    pub static ref RE_SYMB_HASH: Regex = Regex::new("([^#\n])#").unwrap();
    pub static ref RE_SYMB_CII: Regex = Regex::new("([^\\[])\\^").unwrap();
    pub static ref RE_SYMB_TILDE: Regex = Regex::new("~").unwrap();
    pub static ref RE_SYMB_DOT_4: Regex = Regex::new("::::").unwrap();
    pub static ref RE_SYMB_COLON_2: Regex = Regex::new("\n::(.*?)::\n").unwrap();
    pub static ref RE_SYMB_COLON_2_INLINE: Regex = Regex::new("::(.*?)::").unwrap();
    pub static ref RE_SYMB_BSLASH: Regex = Regex::new("\\\\").unwrap();
    pub static ref RE_SYMB_BSLASH2: Regex = Regex::new("\\\\textbackslash \\\\textbackslash ").unwrap();
    pub static ref RE_SYMB_FI: Regex = Regex::new("fi").unwrap();
    pub static ref RE_CHAR_I_DOTLESS: Regex = Regex::new("I").unwrap();
    pub static ref RE_CHAR_MINOR_I_DOTTED: Regex = Regex::new("i").unwrap();
    pub static ref RE_CHAR_DOT_DOT: Regex = Regex::new("̇̇").unwrap(); // two consecutive dots (from dotted i̇i̇)
    pub static ref RE_CHAR_CJK_COLON: Regex = Regex::new("([^\\d+])：").unwrap();
    pub static ref RE_PATT_FOOT_DEF: Regex = Regex::new("(?m)^\\[\\^\\d+\\]:").unwrap();
    pub static ref RE_PATT_FOOT_ZERO: Regex = Regex::new("(?m)^\\[\\^0\\]: (.*?)$").unwrap(); // zero'th footnote. It's anonymous but required. TODO: support multi-line
    pub static ref RE_PATT_FOOT_ANON: Regex = Regex::new("(?m)^\\[\\^\\]: (.*?)$").unwrap(); // anon footnote. not original, then not required. TODO: support multi-line
    pub static ref RE_PATT_FOOT_CHAR: Regex = Regex::new("(?m)^\\[\\^\\D\\]: (.*?)$").unwrap(); // translator footnote. not original, then not required. TODO: support multi-line
    pub static ref RE_PATT_FOOT_DEF_CONT: Regex = Regex::new("(?m)^    ").unwrap();

    // pub static ref RE_PATT_HASH_BEFORE_UTFBOX: Regex = Regex::new("(#.*\\n)\\n\\\\utfbox").unwrap();
    // pub static ref RE_PATT_PART_BEFORE_UTFBOX: Regex = Regex::new("(\\\\part\\{.*\\}\\n)\\n\\\\utfbox").unwrap();
    // pub static ref RE_PATT_WHITE_BEFORE_UTFBOX: Regex = Regex::new("\\s*\\\\utfbox").unwrap();

    // temporary
    pub static ref RE_SYMB_UNDERSCORE: Regex = Regex::new("_").unwrap();
}
