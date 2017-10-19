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
//extern crate walkdir;


mod errors {
    error_chain!{}
}
use errors::*;

//use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::fs::read_dir;
use std::path::Path;
//use std::io::prelude::*;
use tera::Tera;

use semver::Version;
use regex::Regex;
//use walkdir::WalkDir;
//use log::Level;

use std::collections::HashMap;
use std::collections::HashSet;



#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct Info {
    language: String,
    fallback: Option<String>,
    translation: bool,
    // cover
    titles: Vec<String>,
    authors: Option<Vec<String>>,
    collaborators: Option<Vec<String>>,
    thanks: Option<Vec<String>>,
    translators: Option<Vec<String>>,
    reviwers: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    // urls
    discussions: Option<Vec<Vec<String>>>,
    transifex_other: Option<String>,
    transifex_done: Option<String>,
    original: Option<String>,
    more_infos: Option<Vec<Vec<String>>>,
    tags_prefix: Option<String>,
    artists: Option<Vec<Vec<String>>>,
    // settings
    reset_footer_active: bool,
    reset_footer_depth: u8,
    clear_page_active: bool,
    clear_page_depth: u8,
    toc_depth_book: u8,
    toc_depth_article: u8,
    toc_depth_mobile: u8,
    targets: Vec<String>,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Consts {
    min_ver: String,
    all_langs: Vec<Lang>,
    transifex_folder_path: String,
}

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = compile_templates!("templates/**/*");
        tera.autoescape_on(vec![".tex"]);
        //tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
    pub static ref RE_FORWARD_ARROW: Regex = 
        Regex::new("\\{->").unwrap();
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Lang {
    short: String,
    long: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Defaults {
    info: Info,
    target: String,

    all_langs: Vec<Lang>,
    def_lang: Lang,
    fall_lang: Option<Lang>,
    other_langs: Vec<Lang>,
}

fn run() -> Result<()> {
    let ymlc = File::open("const.yml")
        .chain_err(|| "Failed to open the yml const file")?;
    let consts: Consts = serde_yaml::from_reader(ymlc)
        .chain_err(|| "Failed to parse the yml const file contents")?;
    let min_ver = Version::parse(&consts.min_ver)
        .chain_err(|| format!("Failed to parse the consts version ({})", &consts.min_ver))?;


    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    struct DirInfo {
        dir: String,
        info: Info,
    };

    // There are several 2D vectors, according to the language and then index. 
    // First, there are the originals and the translations 2D vectors.
    // Then each one is separated into the ones that uses transifex (_tsfx), and those who don't (_local).

    // Then, regarding transifex, a relationship between the originals and translations is needed.
    //   since a thai translation might have come from english, which might have come from japanese, the actual original text,
    //   the relationship is not straightforward. Each text should point at the other two.
    // So two hashmaps are built. On both of them, the key is the transifex 'done' url.
    //   In the first hashmap the value is a copy of the Info structure itself
    //   In the second hashmap the value is a vector of 'done' urls (other keys) - this is cheap to copy.
    //   So for a given 'done' url key, we can access it's Info structure and also the related translation projects Info structures.

    // Then for each project, the script will work on it's 'tmp' folder, so the original contents arent touched.
    // They are actually copied into tmp/original/ folder, to make things simpler.
    // Then inside tmp/ folder, a folder for each target is created, with the tmp/original/ contents.
    // So each target may work on the files isolated from other projects and from other targets.

    // TODO: also build the projects that are _local (not transifex related).
    // TODO: test projects that are translations and are linked to their original language, but aren't finished.
    //   maybe: basically consider unfinished translations as finished and include the progress info accordingly.

    let dir_info_hold: Vec<DirInfo> = vec![];
    let (originals, translations): (Vec<Vec<DirInfo>>, Vec<Vec<DirInfo>>) = 
        consts.all_langs.iter().filter_map(|lang_dir| {
            info!("Reading language directory: {}", lang_dir.short);
            let dir = fs::read_dir(format!("{}/from_{}/", consts.transifex_folder_path, lang_dir.short));

            if let Err(e) = dir {
                warn!("Failed to open the contents of from_{} directory. Error: {}", lang_dir.short, e);
                return None;
            }
            let (oks, errs): (Vec<Result<DirInfo>>, Vec<Result<DirInfo>>) = dir.unwrap().into_iter().map(|proj_dir| {
                let proj_dir = proj_dir
                    .map_err(|e| format!("Failed to open language directory {} due to {}", lang_dir.short, e))?
                    .path();
                let proj_dir = proj_dir.display();
                let yml = File::open(format!("{}/info.yml", proj_dir))
                    .map_err(|e| format!("Failed to open the yml info file in folder {}. Error: {}", proj_dir, e))?;
                let info: Info = serde_yaml::from_reader(yml)
                    .map_err(|e| format!("Failed to parse the yml info file contents in folder {}. Error: {}", proj_dir, e))?;
                let info_ver = Version::parse(&info.version)
                    .map_err(|e| format!("Failed to parse the info version ({}). Error: {}", &info.version, e))?;
                if info_ver > min_ver {
                    bail!(format!("Error: version of info yaml file is too low ({} < {})", &info_ver, min_ver));
                }

                let dir_info = DirInfo{
                    dir: format!("{}", &proj_dir),
                    info: info,
                };

                Ok(dir_info)
            }).partition(|x: &Result<DirInfo>| x.is_ok());
            for e in errs {
                warn!("project not read: {}", e.err().unwrap());
            }
            if let None = oks.iter().next() {
                None
            } else {
                Some(oks.into_iter().collect::<Result<Vec<DirInfo>>>().unwrap().into_iter()
                    .partition(|dir_info| !dir_info.info.translation))
            }
    }).unzip();

    // further separate originals into those that have transifex urls and those that dont
    let (originals_tsfx, originals_local): (Vec<Vec<DirInfo>>, Vec<Vec<DirInfo>>) = originals.into_iter().map(|lang| {
        lang.into_iter().partition(|p| p.info.transifex_done.is_some())
    }).unzip();
    
    // to the same for translations
    let (translations_tsfx, translations_local): (Vec<Vec<DirInfo>>, Vec<Vec<DirInfo>>) = translations.into_iter().map(|lang| {
        lang.into_iter().partition(|p| p.info.transifex_other.is_some())
    }).unzip();
    // note: tsfx may be partial (no transifex_done), therefore it wont be listed in the other project.
    // TODO: a 'preview' notice could be added to this file cover, since its not fully translated

    let insert_into_hm = |(mut hm_s, mut hm_di): (HashMap<String, HashSet<String>>, HashMap<String, DirInfo>), lang: &Vec<DirInfo>| {
        for dir_info in lang {
            let di: &DirInfo = dir_info;
            let itself: Option<String> = di.info.transifex_done.clone();
            if let None = itself {
                continue;
            }
            let ref itself = itself.unwrap();
            if let Some(old) = hm_s.get(itself) {
                panic!("Error: repeated originals_tsfx key value.\nThis: {:?}\nAnd this: {:?}\nYou should change the transifex_done.", 
                    old, &dir_info.info);
            }
            hm_di.insert(itself.clone(), di.clone());
            let mut hs_s = HashSet::new();
            hs_s.insert(itself.clone());
            hm_s.insert(itself.clone(), hs_s);
        }
        (hm_s, hm_di)
    };

    let mut tsfx_str: HashMap<String, HashSet<String>> = HashMap::new();
    let mut tsfx_dirinfo: HashMap<String, DirInfo> = HashMap::new();
    let (mut tsfx_str, mut tsfx_dirinfo) = originals_tsfx.iter().chain(translations_tsfx.iter()).fold((tsfx_str, tsfx_dirinfo), insert_into_hm);

    tsfx_str = translations_tsfx.iter()
        .fold(tsfx_str, |mut hm, lang| {
        for dir_info in lang {
            let di = dir_info;
            let itself = di.info.transifex_done.clone();
            let ref other = di.info.transifex_other.clone().unwrap();
            if let None = itself {
                continue;
            }
            let ref itself = itself.unwrap();
            hm = mutually_add(hm, itself, other);
        }
        hm
    });
    

    fn mutually_add (mut hm: HashMap<String, HashSet<String>>, a: &str, b: &str) 
        -> HashMap<String, HashSet<String>> {
        let a_ref = hm.get(a).clone().unwrap().clone();
        let b_ref = hm.get(b).clone().unwrap().clone();
        let union: HashSet<String> = HashSet::new();
        let union: HashSet<&String> = union.union(&a_ref).collect(); 
        let union: HashSet<String> = union.into_iter().map(|x| x.clone()).collect();
        let union: HashSet<&String> = union.union(&b_ref).collect(); 
        let union: HashSet<String> = union.into_iter().map(|x| x.clone()).collect();
        if a != b {
            if let Some(a_set) = hm.get_mut(a) {
                *a_set = union.clone();
            }
            if let Some(b_set) = hm.get_mut(b) {
                *b_set = union.clone();
            }
            for e in &a_ref {
                hm = mutually_add(hm, a, e);
            }
            for e in &b_ref {
                hm = mutually_add(hm, b, e);
            }
        } 
        hm
    } 

    println!("\n{:?}\n", &tsfx_str);
    println!("\n{:?}\n", &tsfx_dirinfo);


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

    'outer: for (key, proj) in tsfx_dirinfo {
        info!("Working on project of key: {}; \nproj: {:?}\n", &key, &proj);
        // clear
        let path = format!("{}/tmp", proj.dir);
        if Path::new(&path).exists() {
            if let Err(e) = fs::remove_dir_all(&path) {
                warn!("Failed to clear the contents of {}/tmp directory. Error: {}", proj.dir, e);
                continue 'outer;
            }
        }


        if let Err(e) = copy_files_except_tmp(&proj.dir, &format!("{}/tmp/original", &proj.dir)) {
            warn!("error: {}", e);
            for e in e.iter().skip(1) {
                info!("caused by: {}", e);
            }
            continue 'outer;
        }

        for target in proj.info.targets {
            if let Err(e) = copy_files_except_tmp(&format!("{}/tmp/original", &proj.dir), &format!("{}/tmp/{}", &proj.dir, target)) {
                warn!("error: {}", e);
                for e in e.iter().skip(1) {
                    info!("caused by: {}", e);
                }
                continue 'outer;
            }
        }

    }
    
    



    info!("finishing..");
    bail!("finished..");
    //Ok(())

    /*
    let mut def = {
        let all_langs = consts.all_langs;
        let (def_lang, other_langs) : (Vec<Lang>, Vec<Lang>) =
            all_langs.iter().cloned().partition(|lang| lang.short == info.language);
        let def_lang: Lang = def_lang.into_iter().next()
            .chain_err(|| "Failed to get the default language information from preset")?;
        let (fall_lang, other_langs) = match info.fallback {
            Some(ref fallback) => {
                let (fall_lang, other_langs) : (Vec<Lang>, Vec<Lang>) = 
                other_langs.into_iter().partition(|lang| &lang.short == fallback);
                (fall_lang.first().cloned(), other_langs)
            },
            None => (None, other_langs),
        };
        Defaults {
            info: info,
            target: "".to_string(),
            all_langs: all_langs,
            def_lang: def_lang,
            fall_lang: fall_lang,
            other_langs: other_langs,
        }
    };

    let base_path = format!("{}/from_{}", consts.transifex_folder_path, info.language);
    for target in info.targets {
        def.target = target.clone();

        // create folder structure
        let tmp_path = format!("{}/tmp/{}", base_path, def.target);
        // create folders..
        // copy everything from parent, except folder "output"

        let mut rendered = TERA.render("test.tex", &def)
            .chain_err(|| "Failed to render the tex templates")?;
        rendered = RE_FORWARD_ARROW.replace_all(&rendered, "{").to_string();
        print!("{}", rendered);

        let mut mdok = File::create("test_ok.tex")
            .chain_err(|| "Falied to create markdown file")?;
        mdok.write_fmt(format_args!("{}", rendered))
            .chain_err(|| "Failed to write on markdown file")?;

    }



    Ok(())
    */
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

