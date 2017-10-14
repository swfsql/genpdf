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
//use std::io::prelude::*;
use tera::Tera;

use semver::Version;
use regex::Regex;
//use walkdir::WalkDir;
//use log::Level;


#[derive(Serialize, Deserialize, Debug, Clone)]
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


    // open cache
    //  if none: find all folders and build cache

    /*
    hm () -> ();
      "nome de cada pasta"
      "link discussao de cada pasta" -> pode repetir?

      como relacionar uma pasta com a outra?
      cada uma ou é original (tem o tfx_new) ou é tradução (tem o tfx_old e talvez o tfx_new)


        <tfx_done> -> <ref struct; hashmap[others done]>
        digamos.. passei por um original; criado um novo. Daí passo por um other done1, fica:
            <first> -> <ref struct; hashmap[second]>
            <second> -> <ref struct; hashmap[first]>
        digamos.. daí criei um novo, que veio do segundo:
            <first> -> <ref struct; hashmap[second]>
            <second> -> <ref struct; hashmap[first]>
            <third> -> <ref struct; hashmap[second, first]> // na hora de adicionar o segundo, adiciona todos que ele tem.. recursivamente
            // mas também adiciona ele mesmo em todos que ele adicionou
            <second> -> <ref struct; hashmap[first, third]>
            <first> -> <ref struct; hashmap[second, third]>

      - não é tradução
        : translation: false
        - não usa o tfx
        - usa o tfx
          : tfx_other = None
          : tfx_done = tfx_done -> chave importante -> do original

      - é tradução
        : translation: true

        - não usa o tfx
          : tfx_other = None 
          : tfx_done = None 

        - usa o tfx
          : tfx_other = tfx_other -> chave importante -> do original relativo na outra língua
          : tfx_done = tfx_done -> chave importante -> do novo na nova língua

        sobre o original: listar outros done;
        sobre outros done: listar o original relativo e outros done.

    */

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct DirInfo {
        dir: String,
        info: Info,
    };

    // planning to relate different documents according to their transifex directory
    // first partitionate them into those that are not translated, and those who are
    // originals contains vectors for each language. For each one, theres a vector of original (non-translation) projects
    // translations contains vectors for each language. For each one, theres a vector of translation projects.
    let (originals, translations): (Vec<Vec<DirInfo>>, Vec<Vec<DirInfo>>) = 
        consts.all_langs.iter().filter_map(|lang_dir| {
            info!("Reading language directory: {}", lang_dir.short);
            let dir = fs::read_dir(format!("{}/from_{}", consts.transifex_folder_path, lang_dir.short));

            if let Err(e) = dir {
                warn!("Failed to open the contents of from_{} directory. Error: {}", lang_dir.short, e);
                return None;
            }
            let (oks, errs): (Vec<Result<DirInfo>>, Vec<Result<DirInfo>>) = dir.unwrap().into_iter()
                .map(|proj_dir| {
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
                    
                    Ok(DirInfo{
                        dir: proj_dir.to_string(),
                        info: info,
                    })
                })
                .partition(|x: &Result<DirInfo>| x.is_ok());
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

    // each project will be accessible with its transifex url. With that, it will be possible to access
    // alternative languages translations. to facilitate template usage, the information will get quite repetitive

    // further separate originals into those that have transifex urls and those that dont
    let (originals_tsfx, originals_local): (Vec<Vec<DirInfo>>, Vec<Vec<DirInfo>>) = originals.into_iter().map(|lang| {
        lang.into_iter().partition(|p| p.info.transifex_done.is_some())
    }).unzip();
    
    // to the same for translations
    let (translations_tsfx, translations_local): (Vec<Vec<DirInfo>>, Vec<Vec<DirInfo>>) = translations.into_iter().map(|lang| {
        lang.into_iter().partition(|p| p.info.transifex_other.is_some())
    }).unzip();
    // note: tsfx may be partial (no transifex_done), therefore it wont be listed in the other project.
    
    



    bail!("chegou..");
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

