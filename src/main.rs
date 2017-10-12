#![recursion_limit = "1024"]
#[macro_use]
extern crate tera;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate serde_yaml;
extern crate semver;


mod errors {
    error_chain!{}
}
use errors::*;

//use std::collections::BTreeMap;
use std::fs::File;
//use std::io::prelude::*;
use tera::Tera;

use semver::Version;


#[derive(Serialize, Deserialize, Debug)]
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
    transifex: Option<String>,
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
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Consts {
    min_ver: String,
    all_langs: Vec<Lang>,
}

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = compile_templates!("templates/**/*");
        tera.autoescape_on(vec![".tex"]);
        //tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Lang {
    short: String,
    long: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Defaults {
    info: Info,
    all_langs: Vec<Lang>,
    def_lang: Lang,
    fall_lang: Option<Lang>,
    other_langs: Vec<Lang>,
}

fn run() -> Result<()> {
    // Load and read yaml file
    let yml = File::open("info.yml")
        .chain_err(|| "Failed to open the yml info file")?;
    let info: Info = serde_yaml::from_reader(yml)
        .chain_err(|| "Failed to parse the yml info file contents")?;
    let ymlc = File::open("const.yml")
        .chain_err(|| "Failed to open the yml const file")?;
    let consts: Consts = serde_yaml::from_reader(ymlc)
        .chain_err(|| "Failed to parse the yml const file contents")?;
    
    let info_ver = Version::parse(&info.version)
        .chain_err(|| format!("Failed to parse the info version ({})", &info.version))?;
    let min_ver = Version::parse(&consts.min_ver)
        .chain_err(|| format!("Failed to parse the consts version ({})", &consts.min_ver))?;
    if info_ver > min_ver {
        bail!("Error: version of info yaml file is too low");
    }

    let def = {
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
            all_langs: all_langs,
            def_lang: def_lang,
            fall_lang: fall_lang,
            other_langs: other_langs,
        }
    };


    let rendered = TERA.render("test.tex", &def)
        .chain_err(|| "Failed to render the tex templates")?;
    print!("{}", rendered);
    Ok(())
}

fn main() {
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

