#![recursion_limit = "1024"]
#[macro_use]
extern crate tera;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate serde_yaml;
extern crate semver;
extern crate regex;
extern crate rayon;
extern crate image;

use image::{FilterType, GenericImage, Pixel, ImageBuffer, imageops};
//extern crate walkdir;

extern crate glium;
#[macro_use]
extern crate imgui;
extern crate imgui_glium_renderer;


mod errors {
    error_chain!{}
}
use errors::*;

//use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;
//use std::io::prelude::*;
use std::io;
use tera::Tera;

use semver::Version;
use regex::Regex;
//use walkdir::WalkDir;
//use log::Level;

use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;

use std::process::Command;
//use std::ffi::OsStr;

use rayon::prelude::*;
use std::env;

use imgui::*;
mod support;


type VS = Vec<String>;
type OVS = Option<Vec<String>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct InfoTranslation {
    language: String,
    is_translation: bool,
    this_project_url: Option<String>,
    fetch_translators: bool,
    fetch_reviwers: bool,
    fetch_progress: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct InfoPerson {
    identifier: Option<String>,
    rule: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct InfoResource {
    rule: Option<String>,
    content: Option<String>,
    websites: OVS,
    description: Option<String>,
    persons: OVS,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct InfoCover {
    cover_file: String,
    cover_dimensions: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct InfoTarget {
    name: String,
    size: String,
    reader: String,
    reset_footer_active: bool,
    reset_footer_depth: u8,
    clear_page_active: bool,
    clear_page_depth: u8,
    has_parts: bool,
    frontmatter_depth: u8,
    toc_depth: u8,
    mini_toc_active: bool,
    mini_toc_depth: u8,
    mini_toc_sec_active: bool,
    mini_toc_sec_depth: u8,
    covers: Vec<InfoCover>,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct Info {
    version: String,
    translation: InfoTranslation,
    titles: VS,
    discussions: Option<Vec<VS>>,
    more_infos: Option<Vec<VS>>,
    tags: OVS,
    tag_prefix: Option<String>,
    persons: Option<Vec<InfoPerson>>,
    resources: Option<Vec<InfoResource>>,
    targets: Vec<InfoTarget>,
    content_files: Vec<VS>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InfoPerson2 {
    name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Info2 {
    authors: Vec<InfoPerson2>,
    translators: Vec<InfoPerson2>,
    collaborators: Vec<InfoPerson2>,
    thanks: Vec<InfoPerson2>,
    reviewers: Vec<InfoPerson2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Consts {
    min_ver: String,
    passages: u8,
    cover_nodes: Vec<String>,
    all_langs_from_dir: String,
    all_langs_to_dir: String,
    output_dir: String,
    initials: Vec<VS>,
    num_cpu: u8,
    all_langs: Vec<Lang>,
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
    pub static ref RE_SYMB_FI: Regex = Regex::new("fi").unwrap(); 
    pub static ref RE_CHAR_I_DOTLESS: Regex = Regex::new("I").unwrap(); 
    pub static ref RE_CHAR_i_DOTTED: Regex = Regex::new("i").unwrap(); 
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


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Lang {
    from_active: bool,
    to_active: bool,
    to_dir_name: String, // pt-BR
    set_lang: String, // brazil (xelatex)
    title: String, // Portuguese (Brazilian)
    from_url: Option<String>, // https://crowdin.com/project/ancap-ch/
    from_dir_name: Option<String>, // from_en
}

#[derive(Serialize, Deserialize, Debug)]
struct Defaults {
    info: Info,
    info2: Info2,
    target: String,
    info_target: InfoTarget,

    sent_initial: String,

    all_langs: Vec<Lang>,
    def_lang: Lang,
    other_langs: Vec<Lang>,

    consts: Consts,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct DirInfo {
    base_dir: String,
    from_dir: String,
    lang_dir: String,
    proj_dir: String,
    info: Info,
}

impl DirInfo {
    fn fulldir(&self) -> PathBuf {
        Path::new(&self.base_dir).join(&self.from_dir).join(&self.lang_dir).join(&self.proj_dir)
    }
    fn fulldir_str(&self) -> String {
        format!("{}/{}/{}/{}", self.base_dir, self.from_dir, self.lang_dir, self.proj_dir)
    }
}

fn run() -> Result<()> {


    let ymlc = File::open("const.yml")
        .chain_err(|| "Failed to open the yml const file")?;
    let consts: Consts = serde_yaml::from_reader(ymlc)
        .chain_err(|| "Failed to parse the yml const file contents")?;
    let min_ver = Version::parse(&consts.min_ver)
        .chain_err(|| format!("Failed to parse the consts version ({})", &consts.min_ver))?;

    env::set_var("RAYON_RS_NUM_CPUS", format!("{}", consts.num_cpu));


    let active_to_langs = consts.all_langs.iter().fold(HashSet::new(), |mut hs, l| {
        if l.to_active {
            hs.insert(&l.to_dir_name);
            hs
        } else {
            hs
        }
    });
    println!("<{:?}>", active_to_langs);


    // let (originals, translations): (Vec<Vec<DirInfo>>, Vec<Vec<DirInfo>>) = 
    let dirs: Vec<DirInfo> = 
        consts.all_langs.iter().filter_map(|lang| {
            // println!("::lang\n{:?}", lang);
            if !lang.from_active { 
                return None; 
            }

            let from_dir_name = lang.from_dir_name.clone();
            if let None = from_dir_name {
                return None;
            }
            let from_dir_name = from_dir_name.unwrap();
            

            info!("Reading language directory: {}", lang.to_dir_name);
            let dir = fs::read_dir(format!("{}/{}", &consts.all_langs_to_dir, &from_dir_name));

            println!("::dir\n{:?}... {:?}", dir, format!("{}/{}", consts.all_langs_to_dir, &from_dir_name));

            if let Err(e) = dir {
                warn!("Failed to open the contents of {}/{} directory. Error: {}", &from_dir_name, lang.to_dir_name, e);
                return None;
            }
            let oks: Vec<DirInfo> = dir.unwrap().into_iter().filter_map(|lang_dir| {

                let lang_dir = lang_dir
                    .map_err(|e| format!("Failed to open language directory {} due to {}", lang.to_dir_name, e));
                if let Err(_) = lang_dir {
                    return None;
                }
                let lang_dir = lang_dir.unwrap().path();
                let lang_dir_name = lang_dir.file_name().unwrap().to_string_lossy().to_string();

                if !active_to_langs.contains(&lang_dir_name) {
                    return None;
                }
                

                let proj_dirs = fs::read_dir(lang_dir);
                let dir_infos = proj_dirs.unwrap().into_iter().filter_map(|proj_dir| {

                    let proj_dir = proj_dir.unwrap().path();
                    let proj_dir_name = proj_dir.file_name().unwrap().to_string_lossy().to_string();
                    // println!("::{} ", proj_dir_name);
                    let yml = File::open(proj_dir.join("info.yml"))
                        .map_err(|e| format!("Failed to open the yml info file in folder {}. Error: {}", proj_dir_name, e));
                    if let Err(_) = yml {
                        // println!(" >> yml err");
                        return None;
                    }
                    let yml = yml.unwrap();
                    let info = serde_yaml::from_reader(yml)
                        .map_err(|e| format!("Failed to parse the yml info file contents in folder {}. Error: {}", proj_dir_name, e));
                    if let Err(_) = info {
                        // println!(" >> info err <{}>", e);
                        return None;
                    }
                    let info: Info = info.unwrap();
                    let info_ver = Version::parse(&info.version)
                        .map_err(|e| format!("Failed to parse the info version ({}). Error: {}", &info.version, e));
                    if let Err(_) = info_ver {
                        // println!(" >> ver err");
                        return None;
                    }
                    let info_ver = info_ver.unwrap();
                    if info_ver > min_ver {
                        // bail!(format!("Error: version of info yaml file is too low ({} < {})", &info_ver, min_ver));
                        // println!(" >> min ver err");
                        return None;
                    }

                    let dir_info = DirInfo{
                        base_dir: format!("{}", &consts.all_langs_to_dir),
                        from_dir: format!("{}", &from_dir_name),
                        lang_dir: format!("{}", &lang_dir_name),
                        proj_dir: format!("{}", &proj_dir_name),
                        info: info,
                    };

                    println!("\ninfo:\n{:?}\n", &dir_info.info);

                    return Some(dir_info);

                }).collect::<Vec<_>>();


                Some(dir_infos)
            }).fold(vec![], |mut vs, v| {
                vs.extend(v);
                vs
            });
            // for e in errs {
            //     warn!("project not read: {}", e.err().unwrap());
            // }
            if let None = oks.iter().next() {
                None
            } else {
                Some(oks.into_iter().collect::<Vec<DirInfo>>()
                    // .into_iter()
                    // .partition(|dir_info| !dir_info.info.translation)
                )
            }
    }).fold(vec![], |mut vs, v| {
        vs.extend(v);
        vs
    });

    //println!("\n\n\n{:?}\n\n\n", dirs);



    fn copy_files_except_tmp(from: &str, to: &str) -> Result<()> {
        fs::create_dir_all(to)
            .chain_err(|| format!("Failed to create a new {} directory.", to))?;

        let dir = Path::new(from);
        let dirs = read_dir(&dir)
            .chain_err(|| format!("Failed to start copying {} contents into the tmp directory.", from))?;

        for path in dirs {
            let path = path
                .chain_err(|| format!("Failed to open a file."))?;
            if path.path().ends_with("tmp") {
                continue;
            }
            let dst = Path::new(to).join(path.path().file_name().unwrap());
            let meta = path.metadata()
                .chain_err(|| format!("Failed to read {} metadata.", path.path().display()))?;
            if meta.is_dir() {
                fs::create_dir(&dst)
                    .chain_err(|| format!("Failed to create a new {:?} directory.", &dst))?;
            } else {
                let orig = path.path();
                fs::copy(&orig, &dst)
                    .chain_err(|| format!("Failed to copy {:?} into {:?} folder.", &orig, &dst))?;
            }
        }
        Ok(())
    }

    fn chk_footnote_proj(proj: &DirInfo, original: &DirInfo) -> Result<Option<Vec<usize>>> {

        let count_foots = |dir: &DirInfo| {
            let ret = dir.info.content_files
                .iter()
                .map(|vs| vs[0].clone())
                .map(|md| {
                    let mut file = File::open(format!("{}/{}", dir.fulldir_str(), md)).unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    // TODO: try using scan again >_> damn you lifetimes
                    let mut foots = vec![];
                    let mut foot = false;
                    for line in contents.lines().map(|line| line.trim()).filter(|line| line.len() != 0) {
                        let line = RE_SYMB_DOT_4.replace_all(&line, "    ").to_string();
                        // println!("{}", &line[0..2]); // may panic
                        if RE_PATT_FOOT_DEF.is_match(&line) {
                            foots.push(1u8);
                            foot = true;
                        } else if foot && RE_PATT_FOOT_DEF_CONT.is_match(&line) {
                            let len = foots.len();
                            foots[len - 1] += 1u8;
                        } else {
                            foot = false;
                        }
                    }
                    (md, foots)
                })
                .collect::<Vec<(_, Vec<_>)>>();
            println!("foots: <{:?}>", &ret);
            ret
        };
        let diff_pos = count_foots(original).iter()
            .zip(count_foots(proj))
            .enumerate()
            .inspect(|&(index, (&(ref md, ref foots_a), (_, ref foots_b)))| {
                println!(" {}: [{}]", index, md);
                foots_a.iter()
                    .zip(foots_b)
                    .inspect(|&(num_a, num_b)| {
                        let diff = if num_a != num_b {" ~"} else {""};
                        println!("  {} == {}{}", num_a, num_b, diff);
                    })
                    .collect::<Vec<_>>();

            })
            .filter(|&(_index, (&(ref md, ref foots_a), (_, ref foots_b)))| {
                foots_a.iter()
                    .zip(foots_b)
                    .any(|(num_a, num_b)| num_a != num_b)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let diff_pos = if diff_pos.is_empty() {None} else {Some(diff_pos)};
        Ok(diff_pos)
    }

    fn gen_proj(proj: &DirInfo, consts: &Consts) -> Result<()> {
        info!("Working on project: {:?}\n", &proj);

        // if skip_templates && proj.proj_dir == "template" 
        if true && proj.proj_dir == "template" {
            return Ok(());
        }

        copy_files_except_tmp(&proj.fulldir_str(), &format!("{}/tmp/original", &proj.fulldir_str()))
            .map_err(|e| format!("Error when copying files into {}/tmp/dir folder. Due to {}.", &proj.fulldir_str(), e))?;

        println!("res: <{:?}>", proj.info.resources);
        if let &Some(ref ress) = &proj.info.resources {
            for res in ress {
                if let Some(ref rule) = res.rule {
                    if rule == "front_cover" {
                        if let Some(ref content) = res.content {
                            let origin = &format!("{}/{}/_asset/_image/{}", &proj.base_dir, &proj.from_dir, content);
                            let dest = &format!("{}/tmp/original/{}", &proj.fulldir_str(), content);
                            println!("antes de copiar");
                            fs::copy(&format!("{}", &origin), &format!("{}", &dest))
                                    .chain_err(|| format!("Error when copying files from {} into {}.", 
                                        &origin, &dest))?;
                            println!("depois de copiar");
                        }
                    }
                }
            }
        }

        let mut authors: Vec<InfoPerson2> = vec![];
        println!("authors: <{:?}>", &proj.info.persons);
        if let &Some(ref persons) = &proj.info.persons {
            for p in persons {
                if let &Some(ref rule) = &p.rule {
                    if rule == "author" {
                        let person = InfoPerson2{
                            name: p.identifier.clone().unwrap(),
                        };
                        authors.push(person);
                    }
                }
            }
        }
        println!("authors: <{:?}>", &authors);

        // lang information
        let all_langs = consts.all_langs.clone();
        let (def_lang, other_langs) : (Vec<Lang>, Vec<Lang>) =
            all_langs.iter().cloned().partition(|lang| lang.to_dir_name == proj.info.translation.language);
        let def_lang: Lang = def_lang.into_iter().next()
            .chain_err(|| "Failed to get the default language information from preset")?;

        // TODO: other translations information (to link among themselves)

        for target in proj.info.targets.clone() {
            let destination = format!("{}/tmp/{}", &proj.fulldir_str(), target.name);
            copy_files_except_tmp(&format!("{}/tmp/original", &proj.fulldir_str()), &destination)
                .map_err(|e| format!("Error when copying files from {}/tmp/original into {}/tmp/{}. Due to {}.", 
                    &proj.fulldir_str(), &proj.fulldir_str(), target.name, e))?;
            
            println!("target: <{:?}>", target);
            // TODO: crop and/or resize the cover images, and replace them

            for cover in &target.covers {
                let img_filepath = format!("{}/{}", &destination, cover.cover_file);
                let mut crop;
                {
                    let mut img = image::open(&img_filepath)
                        .map_err(|e| format!("Error when opening image file from {}. Due to {}.", &img_filepath, e))?;
                    let (offsetx, offsety) = (cover.cover_dimensions[0], cover.cover_dimensions[1]);
                    let (width, height) = img.dimensions();
                    let width = if cover.cover_dimensions[2] == 0 {width - offsetx} else {cover.cover_dimensions[2]};
                    let height = if cover.cover_dimensions[3] == 0 {height - offsety} else {cover.cover_dimensions[3]};
                    // TODO: add paper proportion measure, so we can crop exceding width or exceding height
                    crop = imageops::crop(&mut img, offsetx, offsety, width, height).to_image();
                }
                crop.save(&img_filepath)
                    .map_err(|e| format!("Error when saving image file to {}. Due to {}.", &img_filepath, e))?;
            }
            
            println!("Next file is <{}>, for the target <{}>. continue? [Y/n] ", &proj.fulldir_str(), &target.name);
            
            let mut initial = 
                if target.name == "article" {false} 
                else if target.name == "book" {true}
                else {false};
            let mut skip_initial = false;
            let mut sec_active = vec![false; 10];

            // let initial_rank = 
            // "ABCDEFGHIJKLMNOPQRSTUVWZ" // ZallmanI
            // "ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÑÒÔÕÖŒÙÚÛÜ" // AM Intex


            // initial
            let mut used_initials = vec![];
            let initials = consts.initials.iter()
                .map(|vs| HashSet::from_iter(vs[1].chars()))
                .collect::<Vec<HashSet<_>>>();
            
            // utfbox, endsecs and resetfoots
            let mut box_clear_foot = vec![];
            
            // let mut s_all = String::new();
            // let path_all = format!("{}/all.tex", &destination);
            // let mut file_all = File::open(&path_all)
            //     .map_err(|e| format!("failed to open content file to replace by regex. Error: {:?}. Path: <{}>", e, &path_all))?;

            // substitute content characters
            for (index, content) in proj.info.content_files.iter().map(|c| &c[0]).enumerate() {
                let path = format!("{}/{}", &destination, &content);
                box_clear_foot.push((false, false, false));

                let mut file = File::open(&path)
                    .map_err(|e| format!("failed to open content file to replace by regex. Error: {:?}. Path: <{}>", e, &path))?;
                let mut s = String::new();
                file.read_to_string(&mut s)
                    .map_err(|e| format!("failed to read content file to replace by regex. Error: {:?}", e))?;
                file = File::create(&path)
                    .map_err(|e| format!("failed to overwrite content file to replace by regex. Error: {:?}", e))?;
                // let mut s2: String = "".into();

                s = format!("\n{}\n", s); // adds new line around each file
                // so headers on top of files won't break



                //s = RE_TEST_A.replace_all(&s, "b").to_string(); // test

                s = RE_SYMB_BSLASH.replace_all(&s, "\\textbackslash ").to_string();
                if proj.info.translation.language == "tr" {
                    s = RE_SYMB_FI.replace_all(&s, "f\\/i").to_string();
                    s = RE_CHAR_i_DOTTED.replace_all(&s, "i̇").to_string(); // TODO: not all is required, just chap names and opening words
                    s = RE_CHAR_DOT_DOT.replace_all(&s, "̇").to_string();
                }
                // s = RE_SYMB_CURLY_BRACK.replace_all(&s, "\\{").to_string(); // TODO
                // s = RE_SYMB_CURLY_BRACK2.replace_all(&s, "\\}").to_string(); // TODO
                // TODO underline...
                s = RE_SYMB_AMPER.replace_all(&s, "\\&{}").to_string();
                s = RE_SYMB_DOLLAR.replace_all(&s, "\\${}").to_string();
                s = RE_SYMB_PERCENT.replace_all(&s, "\\%{}").to_string();
                // s = RE_SUB_HASH_SPACE_HASH.replace_all(&s, "##").to_string(); // # # -> ## (crowdin messed this up)
                if target.has_parts {
                    println!("start to test part!");
                    // s = RE_SUB_HASH_DOWNGRADE.replace_all(&s, "##").to_string();
                    // pub static ref RE_SUB_HASH_DOWNGRADE: Regex = Regex::new("^#(#*)([^#]*)$").unwrap(); 
                    s = s.lines().map(|ref line| {
                        let caps = RE_SUB_HASH_DOWNGRADE.captures(&line);
                        if let None = caps {
                            return line.to_string() + "\n";
                        }
                        let caps = caps.unwrap();
                        if let None = caps.get(0) {
                            return line.to_string() + "\n";
                        }
                        let c1 = caps.get(1).map_or("", |c| c.as_str().clone());
                        let c2 = caps.get(2).map_or("", |c| c.as_str().clone());
                        if c1.chars().count() > 0 {
                            return format!("{}{}", c1.clone(), c2.clone()).to_string() + "\n";
                        } else {
                            println!("found a part!");
                            if target.clear_page_active {
                                box_clear_foot[index - 1].0 = true;
                                box_clear_foot[index - 1].1 = true;
                                for i in 0..9 {
                                    sec_active[i] = false;
                                }
                            }
                            if target.reset_footer_active {
                                box_clear_foot[index - 1].2 = true;
                            }
                            return format!("\\part{{{}}}", c2.clone()).clone() + "\n";
                        }
                    }).collect::<String>();
                    println!("finished to test part!");
                }
                s = RE_SYMB_HASH.replace_all(&s, "$1\\texthash{}").to_string();
                s = RE_SYMB_CII.replace_all(&s, "$1\\textasciicircum{}").to_string();
                s = RE_SYMB_DOT_4.replace_all(&s, "    ").to_string();
                s = RE_SYMB_TILDE.replace_all(&s, "\\textasciitilde{}").to_string();
                s = RE_CHAR_CJK_COLON.replace_all(&s, "$1:").to_string();
                s = RE_SYMB_COLON_2.replace_all(&s, "$$$ $1 $$$").to_string(); // $1$ ::X:: 
                s = RE_SYMB_COLON_2_INLINE.replace_all(&s, "$$ $1 $$").to_string();
                // TODO: normalize previous replacements inside math-mode


                let mut do_section_clear = |line: &str| {
                    println!("Debug line:\n{}", &line);
                    let depth = line.chars().take_while(|&c| c == '#').count();
                    if depth == 0 {
                        // line.to_string()
                    } else {
                        for i in depth..9 {
                            sec_active[i] = false;
                        }
                        if sec_active[depth - 1] {
                            // let line_start: String = "".into();
                            if target.clear_page_active && depth <= target.clear_page_depth as usize{
                                if target.has_parts && depth - 1 == 0 {
                                    box_clear_foot[index - 1].0 = true;
                                    // line_start += "\\utfbox";
                                } else {
                                    box_clear_foot[index - 1].0 = true;
                                    box_clear_foot[index - 1].1 = true;
                                    // line_start += "\\utfbox\\clearpage";
                                }
                            };
                            if target.reset_footer_active && depth <= target.reset_footer_depth as usize {
                                box_clear_foot[index - 1].2 = true;
                                // line_start += "\\endfoot";
                            }
                            // format!("{}\n\n{}", &line_start, line.to_string())
                        } else {
                            sec_active[depth - 1] = true;
                            // line.to_string()
                        }
                    }
                    ()
                };
                println!("finished endfoot and endsec insertions");

                let mut do_initial = |line: &str, start: &str, inis: &mut Vec<char>| {
                    if line.starts_with(start) {
                        initial = true;
                        skip_initial = false;
                        line.to_string()
                    } 
                    else if line.starts_with("#") {
                        skip_initial = true;
                        line.to_string()
                    }
                    else if initial && !skip_initial {
                        if line.trim() == "" || line.starts_with("[^") {
                            line.to_string()
                        } else {
                            initial = false;
                            let initials: String = line.chars()
                                .take_while(|c| c.is_alphanumeric() && !c.is_numeric() && !c.is_whitespace())
                                .collect();
                            let line_start_start: String = initials.chars().take(1).collect();
                            let line_start_end: String = initials.chars().skip(1).collect();
                            let line_start = format!("\\DECORATE{{{}}}{{{}}}", line_start_start, line_start_end);
                            let line_end: String = line.chars().skip(initials.chars().count()).collect();
                            println!("line start start: <{:?}>", &line_start_start);
                            if let Some(c) = line_start_start.chars().next() {
                                inis.push(c);
                            }
                            
                            format!("{}{}", line_start, line_end)

                        }
                    } else {
                        skip_initial = false;
                        line.to_string()
                    }
                };

                if target.name == "article" {
                    s = s.lines()
                        .map(|line| do_initial(&line, &"# ", &mut used_initials) + "\n")
                        .collect::<String>();
                } else if target.name == "book" {
                    s = s.lines()
                        .map(|line| do_initial(&line, &"# ", &mut used_initials) + "\n")
                        .collect::<String>();
                }

                // section clearing (new page, reset footer)
                if target.reset_footer_active || target.clear_page_active {
                    s.lines().map(|line| do_section_clear(&line)).collect::<()>();
                }

                s = RE_PATT_FOOT_ZERO.replace_all(&s, "\\blfootnote{$1}\n").to_string(); // 
                s = RE_PATT_FOOT_ANON.replace_all(&s, "\\blfootnote{$1}\n").to_string(); // 
                s = RE_PATT_FOOT_CHAR.replace_all(&s, "\\trfootnote{$1}\n").to_string(); // 
                
                // if target.name == "article" {
                //     s = s.lines().map(|line| do_initial(&line, &"# ") + "\n").collect::<String>();
                // } else if target.name == "book" {
                //     s = s.lines().map(|line| do_initial(&line, &"## ") + "\n").collect::<String>();
                // }

                // temporary
                // s = RE_SYMB_UNDERSCORE.replace_all(&s, "*").to_string();
                
                // s2 = "".into(); // loop
                // while s2 != s {
                //     s2 = s;
                //     s = RE_SYMB_AMPER.replace_all(&s, "\\&{}").to_string();
                // }

                // s_all = format!("{}{}", &s_all, &s);
                // s_all = RE_PATT_HASH_BEFORE_UTFBOX.replace_all(&s_all, "\\utfbox\n$1").to_string();
                // s_all = RE_PATT_WHITE_BEFORE_UTFBOX.replace_all(&s_all, "\\utfbox").to_string();


                // file_all.write_all(s_all.as_bytes())
                //     .map_err(|e| format!("failed to write on content file that was replaced by regex. Error: {:?}", e))?;

                file.write_all(s.as_bytes())
                    .map_err(|e| format!("failed to write on content file that was replaced by regex. Error: {:?}", e))?;


            }

            // repalce every file to add footnote reset, the end of chapter box and clearpage information
            for (content, box_clear_foot) 
            in proj.info.content_files.iter()
                .map(|c| &c[0])
                .zip(box_clear_foot.iter()) {
                let path = format!("{}/{}", &destination, &content);

                let mut file = File::open(&path)
                    .map_err(|e| format!("failed to open content file to replace by regex. Error: {:?}. Path: <{}>", e, &path))?;
                let mut s = String::new();
                file.read_to_string(&mut s)
                    .map_err(|e| format!("failed to read content file to replace by regex. Error: {:?}", e))?;
                s.trim();
                
                let box_s = if box_clear_foot.0 {"\\utfbox"} else {""};
                let clear_s = if box_clear_foot.1 {"\\clearpage"} else {""};
                let foot_s = if box_clear_foot.2 {"\\endfoot"} else {""};

                file = File::create(&path)
                    .map_err(|e| format!("failed to overwrite content file to replace by regex. Error: {:?}", e))?;
                file.write_all(format!("\n{}{}{}{}\n", s.trim(), foot_s, box_s, clear_s).as_bytes())
                    .map_err(|e| format!("failed to write on content file that was replaced by regex. Error: {:?}", e))?;

            }
            println!("finished the whole substitutions");

            let used_initials_hs: HashSet<char> = HashSet::from_iter(used_initials);
            let sent_initial = if let Some(pos) = initials.iter().position(|best| best.is_superset(&used_initials_hs)) {
                &consts.initials[pos][0]
            } else {
                ""
            };


            // let authors = proj.info.persons_id.iter().
            let info2 = Info2 {
                authors: authors.clone(),
                translators: vec![],
                collaborators: vec![],
                thanks: vec![],
                reviewers: vec![],
            };
            let def = Defaults {
                info: proj.info.clone(),
                info2: info2.clone(),
                target: target.name.clone(),
                info_target: target.clone(),
                //
                sent_initial: sent_initial.to_string(),
                //
                all_langs: all_langs.clone(),
                def_lang: def_lang.clone(),
                other_langs: other_langs.clone(),
                //
                consts: consts.clone(),
            };

            // if def.info.language != "br" {
            //     continue;
            // }

            println!("start rendering");

            let mut rendered = TERA.render("main.tex", &def)
                .chain_err(|| "Failed to render the tex templates")?;
            rendered = RE_FORWARD_ARROW.replace_all(&rendered, "{").to_string(); // }
            debug!("{}", rendered);

            println!("finished rendering");

            let mut mdok = File::create(format!("{}/tmp/{}/main_ok.tex", &proj.fulldir_str(), target.name))
                .chain_err(|| "Falied to create tex file")?;
            mdok.write_fmt(format_args!("{}", rendered))
                .chain_err(|| "Failed to write on tex file")?;

            info!("TeX file written.");

            let cdpath = fs::canonicalize(format!("{proj}/tmp/{tgt}", proj=&proj.fulldir_str(), tgt=&target.name))
                .chain_err(|| "Failed to canonicalize the working project directory.")?
                .into_os_string().into_string()
                .map_err(|e| format!("Invalid working directory string path. Error: {:?}", e))?;
            //let cmd = format!("xelatex main_ok.tex -include-directory=\"{cd}\" -output-directory=\"{cd}\" -halt-on-error --shell-escape", 
            //let cmd = format!("xelatex \"{cd}\\main_ok.tex\" -halt-on-error --shell-escape", 
            //let cmd = format!("\"cd /d \"{cd}\" && xelatex main_ok.tex -halt-on-error --shell-escape\"", 
            //let cmd = format!("cd ../transifex && ls");
            let cmd = &format!("cd {cd} && xelatex main_ok.tex -halt-on-error --shell-escape",
            //let cmd = OsStr::new(&cmd);
                    cd=&cdpath.replace(" ", "^ ")[4..]);
                    //cd=&proj.dir[2..]);
            println!("Command:\n{:?}", &cmd);
            //println!("Command:\n{}", &cmd);

            //xelatex main_ok.tex -include-directory="C:/Users/Thiago/Desktop/ancap.ch/transifex/from_th/the essay name/tmp/book" -output-directory="C:/Users/Thiago/Desktop/ancap.ch/transifex/from_th/the essay name/tmp/book" -halt-on-error --shell-escape

            for i in 0..consts.passages {
                let output = Command::new("cmd")
                    .args(&["/C", cmd])
                    //.args(&["/C", cmd.to_str().unwrap()])
                    .output()
                    .chain_err(|| "Falied to create tex file")?;
                
                if !output.status.success() {

                    let err_msg = format!("status: {}\n; stdout: {}\n; stderr: {}\n", 
                        output.status, 
                        String::from_utf8_lossy(&output.stdout), 
                        String::from_utf8_lossy(&output.stderr));

                        println!("error when executing xelatex: \n{}", err_msg);

                        bail!("Error {}.", err_msg);
                        // Err(format!("error.. "));
                } else { // success
                    // copy to output folder

                    // output/pt-BR/EEPP/EEPP-pc.pdf
                    // const  lang  name name-target.ext

                    if i != consts.passages - 1 {
                        continue;
                    }

                    println!("preparing to copy a file..");

                    let extension = Path::new(&format!("{}/main_ok.pdf", &destination)).extension().unwrap().to_string_lossy().to_string();

                    let capitals = proj.proj_dir.chars().filter(|c| c.is_uppercase()).collect::<String>();
                    let out_dest_dir = format!("{}/{}/{}", 
                        consts.output_dir, 
                        &def.def_lang.title,
                        &capitals);
                    let out_dest_file = format!("{}-{}.{}", 
                        &proj.proj_dir, target.name, extension);
                    let out_dest = format!("{}/{}", out_dest_dir, out_dest_file);
                    
                    fs::create_dir_all(&out_dest_dir)
                        .chain_err(|| format!("Error when creating directories {}", 
                            &out_dest_dir))?;

                    fs::copy(
                        &format!("{}/main_ok.pdf", &destination), 
                        format!("{}", &out_dest))
                        .chain_err(|| format!("Error when copying files from {}/main_ok.pdf into {}.", 
                            &destination, &out_dest))?;
                    
                    println!("\n->file copied to: \n{}\n", &out_dest);
                }

            }

        }

        Ok(())
    }


    // ui.checkbox(im_str!("With Alpha Preview"), &mut s.alpha_preview);


    {
        const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let dir_by_lang = |ds: Vec<DirInfo>| {
            consts.all_langs.iter().map(|l| {
                let lan = &l.to_dir_name;
                let filtered = ds.iter().cloned()
                    .filter(|d| &d.lang_dir == lan)
                    .map(|d| (d, false))
                    .collect::<Vec<(DirInfo, bool)>>();
                (lan.to_string(), filtered.clone())
            }).collect::<Vec<(_, Vec<_>)>>()
        };


        let mut dirs2 = dir_by_lang(dirs.clone());

        support::run("hellow_world.rs".to_owned(), CLEAR_COLOR, |ui| {
            hello_world(ui, &mut dirs2, consts.clone())
        });




        fn hello_world<'a>(ui: &Ui<'a>, dirs_by_lang: &mut Vec<(String, Vec<(DirInfo, bool)>)>, consts: Consts) -> bool {
            ui.window(im_str!("Hello world"))
                .size((1266.0, 618.0), ImGuiCond::FirstUseEver)
                .build(|| {
                    // ui.text(im_str!("Hello world!"));
                    // ui.text(im_str!("This...is...imgui-rs!"));
                    // ui.separator();
                    // let mouse_pos = ui.imgui().mouse_pos();
                    // ui.text(im_str!(
                    //     "Mouse Position: ({:.1},{:.1})",
                    //     mouse_pos.0,
                    //     mouse_pos.1
                    // ));
                    // ui.separator();
                    if ui.small_button(im_str!("Clear all tmps")) {
                        let clear_d:Vec<DirInfo> = dirs_by_lang.iter().cloned()
                            .map(|(lan, d)| d)
                            .fold(vec![], |mut acc, ref vo12| {
                                let chk_dirs = vo12.iter()
                                    .map(|&(ref dir, checked): &(DirInfo, bool)| 
                                        dir.clone()) 
                                    .collect::<Vec<DirInfo>>();
                                acc.extend(chk_dirs);
                                acc
                            });
                        for proj in &clear_d {
                            let path = proj.fulldir().join("tmp");
                            // let path = format!("{}/tmp", proj.fulldir());
                            if Path::new(&path).exists() {
                                fs::remove_dir_all(&path)
                                    .map_err(|e| format!("Failed to clear the contents of {}/tmp directory. Due to {}.", proj.fulldir_str(), e)); // ?
                            }
                        }
                    }
                    if ui.small_button(im_str!("Check footnotes")) {
                        let originals:HashMap<String,DirInfo> = dirs_by_lang.iter().cloned()
                            // .inspect(|&(ref lan, ref ds)| {
                            //     ds.iter().cloned()
                            //     .inspect(|d| {
                            //         println!("<{}> == <{}>", 
                            //             &d.0.from_dir,
                            //             &format!("from_{}", lan)
                            //         );
                            //     })
                            //     .collect::<Vec<_>>();
                            // })
                            .map(|(lan, ds)| 
                                (
                                    lan.clone(), 
                                    ds.iter().cloned()
                                        .filter(|d| &d.0.from_dir == &format!("from_{}", lan))
                                        .collect::<Vec<_>>()
                                )
                            )
                            .fold(HashMap::new(), |mut acc, (ref lan, ref ds)| {
                                for &(ref d, _) in ds {
                                    println!("insert: <{}>", format!("from_{}_{}", lan, d.proj_dir));
                                    acc.insert(format!("from_{}_{}", lan, d.proj_dir), d.clone());
                                }
                                acc
                            });
                        let chk_d:Vec<DirInfo> = dirs_by_lang.iter().cloned()
                            .map(|(lan, d)| d)
                            .fold(vec![], |mut acc, ref vo12| {
                                let chk_dirs = vo12.iter()
                                    .filter_map(|&(ref dir, checked): &(DirInfo, bool)| 
                                        if checked {
                                            Some(dir.clone())
                                        } 
                                        else {None})
                                    .collect::<Vec<DirInfo>>();
                                acc.extend(chk_dirs);
                                acc
                            });

                        // println!("{:?}", originals);
                        // let dir_res = chk_d.par_iter()
                        let dir_res = chk_d.iter()
                            .map(|proj| {
                                println!("getting: <{}>", &format!("{}_{}", proj.from_dir, proj.proj_dir));

                                (
                                    proj, 
                                    chk_footnote_proj(&proj, 
                                        originals.get(&format!("{}_{}", proj.from_dir, proj.proj_dir)).unwrap()
                                    )
                                )
                            }
                            )
                            .inspect(|&(ref proj, ref res)| 
                                if let &Err(_) = res {
                                    println!("\npeek err: <{}>~~~~~~~~~~\n", proj.fulldir_str());
                                } 
                                else if let &Ok(Some(ref pos)) = res {
                                    println!("\npeek foot diff: <{}>~~~~~~~~~~\n-at contents {:?}", 
                                        proj.fulldir_str(),
                                        pos);
                                }
                            )
                            .filter(|&(_, ref res)| if let &Err(_) = res {true} else {false})
                            .collect::<Vec<(_, Result<_>)>>();
                        println!("Projects that err:\n");
                        for (proj, _res) in dir_res {
                            println!("err: <{}>", proj.fulldir_str());
                        }

                    } else 
                    if ui.small_button(im_str!("Test selected (cover + 3 files)")) {
                        let chk_d:Vec<DirInfo> = dirs_by_lang.iter().cloned()
                            .map(|(lan, d)| d)
                            .fold(vec![], |mut acc, ref vo12| {
                                let chk_dirs = vo12.iter()
                                    .filter_map(|&(ref dir, checked): &(DirInfo, bool)| 
                                        if checked {
                                            let mut dc = dir.clone();
                                            dc.info.content_files = dc.info.content_files.into_iter().take(3).collect();
                                            Some(dc)
                                            } 
                                        else {None})
                                    .collect::<Vec<DirInfo>>();
                                acc.extend(chk_dirs);
                                acc
                            });

                        // let dir_res = chk_d.par_iter()
                        let dir_res = chk_d.iter()
                            .map(|proj| (proj, gen_proj(&proj, &consts)))
                            .inspect(|&(ref proj, ref res)| if let &Err(ref e) = res {
                                println!("\npeek err: <{}>~~~~~~~~~~\n", proj.fulldir_str());
                                // println!("\nerr: {:?}\n", &e);
                                // for ee in e.iter().skip(1) {
                                //     println!("caused by: {:?}", ee);
                                // }
                            } )
                            .filter(|&(_, ref res)| if let &Err(_) = res {true} else {false})
                            .collect::<Vec<(_, Result<_>)>>();
                        println!("Projects that err:\n");
                        for (proj, _res) in dir_res {
                            println!("err: <{}>", proj.fulldir_str());
                        }

                    } else 
                    if ui.small_button(im_str!("Run selected")) {
                        let chk_d:Vec<DirInfo> = dirs_by_lang.iter().cloned()
                            .map(|(lan, d)| d)
                            .fold(vec![], |mut acc, ref vo12| {
                                let chk_dirs = vo12.iter()
                                    .filter_map(|&(ref dir, checked): &(DirInfo, bool)| 
                                        if checked {Some(dir.clone())} 
                                        else {None})
                                    .collect::<Vec<DirInfo>>();
                                acc.extend(chk_dirs);
                                acc
                            });

                        // let dir_res = chk_d.par_iter()
                        let dir_res = chk_d.iter()
                            .map(|proj| (proj, gen_proj(&proj, &consts)))
                            .inspect(|&(ref proj, ref res)| if let &Err(_) = res {
                                println!("\npeek err: <{}>~~~~~~~~~~\n", proj.fulldir_str());
                            } )
                            .filter(|&(_, ref res)| if let &Err(_) = res {true} else {false})
                            .collect::<Vec<(_, Result<_>)>>();
                        println!("Projects that err:\n");
                        for (proj, _res) in dir_res {
                            println!("err: <{}>", proj.fulldir_str());
                        }


                    } else 
                    if ui.small_button(im_str!("Clear all")) {
                        for &mut(ref mut lan, ref mut d2) in dirs_by_lang {
                            for &mut(ref dir, ref mut checked) in d2.iter_mut() {
                                *checked = false;
                            };
                        }
                    } else 
                    if ui.small_button(im_str!("Toggle all")) {
                        for &mut(ref mut lan, ref mut d2) in dirs_by_lang {
                            for &mut(ref dir, ref mut checked) in d2.iter_mut() {
                                *checked = !*checked;
                            };
                        }
                    } else {
                        ui.tree_node(im_str!("Projects")).build(|| for &mut(ref mut lan, ref mut d2) in dirs_by_lang {
                            ui.tree_node(im_str!("{}", lan)).build(|| for &mut(ref dir, ref mut checked) in d2.iter_mut() {
                                // ui.text(im_str!("{}", dir.proj_dir));
                                ui.checkbox(im_str!("{}", dir.proj_dir), checked);
                            });
                        });
                    }
                });

            true
        }
    }

{


    println!("clear all tmp folders? [y/N] ");
    


    // bail!("MORREU MAS PASSA BEM...");

    // TODO: a structure that groups some information for the same project for different languages



}

    info!("finished..");
    Ok(())
}

fn main() {
    env_logger::init().unwrap();
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

