use crate::{consts, dir_info, info};
use failure::{Error, ResultExt};
use image::{imageops, GenericImage};
use std::collections::HashSet;
use std::fs;
use std::fs::read_dir;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::iter::FromIterator;
use std::path::Path;
use std::process::Command;

pub fn clear_tmp(proj: &dir_info::DirInfo) -> Result<(), Error> {
    let path = proj.fulldir().join("tmp");
    // let path = format!("{}/tmp", proj.fulldir());
    if Path::new(&path).exists() {
        std::fs::remove_dir_all(&path).map_err(|e| {
            format_err!(
                "{}",
                fh!(
                    "Failed to clear the contents of {}/tmp directory. Due to {}.",
                    proj.fulldir_str(),
                    e
                )
            )
        })
    } else {
        Ok(())
    }
}

pub fn copy_files_except_tmp(from: &str, to: &str) -> Result<(), Error> {
    fs::create_dir_all(to) //
        .with_context(wfh!("Failed to create a new {} directory.", to))?;

    let dir = Path::new(from);
    let dirs = read_dir(&dir) //
        .with_context(wfh!(
            "Failed to start copying {} contents into the tmp directory.",
            from
        ))?;

    for path in dirs {
        let path = path //
            .with_context(wfh!("Failed to open a file."))?;
        if path.path().ends_with("tmp") {
            continue;
        }
        let dst = Path::new(to).join(
            path.path()
                .file_name()
                .unwrap_or_else(|| panic!("{}", &fh!())),
        );
        let meta = path
            .metadata()
            .with_context(wfh!("Failed to read {} metadata.", path.path().display()))?;
        if meta.is_dir() {
            fs::create_dir(&dst)
                .with_context(wfh!("Failed to create a new {:?} directory.", &dst))?;
        } else {
            let orig = path.path();
            fs::copy(&orig, &dst) //
                .with_context(wfh!("Failed to copy {:?} into {:?} folder.", &orig, &dst))?;
        }
    }
    Ok(())
}

/*
pub fn chk_footnote_proj(
    proj: &dir_info::DirInfo,
    original: &dir_info::DirInfo,
) -> Result<Option<Vec<usize>>, Error> {
    let count_foots = |dir: &dir_info::DirInfo| {
        let ret = dir
            .info
            .content_files
            .iter()
            .map(|vs| vs[0].clone())
            .map(|md| {
                let mut file = File::open(format!("{}/{}", dir.fulldir_str(), md)).unwrap_or_else(|_| panic!("{}", &fh!()));
                let mut contents = String::new();
                file.read_to_string(&mut contents).with_context(wfh!()).unwrap();

                // TODO: try using scan again >_> damn you lifetimes
                let mut foots = vec![];
                let mut foot = false;
                for line in contents
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| line.len() != 0)
                {
                    let line = consts::RE_SYMB_DOT_4.replace_all(&line, "    ").to_string();
                    // println!("{}", &line[0..2]); // may panic
                    if consts::RE_PATT_FOOT_DEF.is_match(&line) {
                        foots.push(1u8);
                        foot = true;
                    } else if foot && consts::RE_PATT_FOOT_DEF_CONT.is_match(&line) {
                        let len = foots.len();
                        foots[len - 1] += 1u8;
                    } else {
                        foot = false;
                    }
                }
                (md, foots)
            })
            .collect::<Vec<(_, Vec<_>)>>();
        dbg!("foots: <{:?}>", &ret);
        ret
    };
    let diff_pos = count_foots(original)
        .iter()
        .zip(count_foots(proj))
        .enumerate()
        .inspect(|&(index, (&(ref md, ref foots_a), (_, ref foots_b)))| {
            dbg!(" {}: [{}]", index, md);
            foots_a
                .iter()
                .zip(foots_b)
                .inspect(|&(num_a, num_b)| {
                    let diff = if num_a != num_b { " ~" } else { "" };
                    dbg!("  {} == {}{}", num_a, num_b, diff);
                })
                .collect::<Vec<_>>();
        })
        .filter(|&(_index, (&(ref md, ref foots_a), (_, ref foots_b)))| {
            foots_a
                .iter()
                .zip(foots_b)
                .any(|(num_a, num_b)| num_a != num_b)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let diff_pos = if diff_pos.is_empty() {
        None
    } else {
        Some(diff_pos)
    };
    Ok(diff_pos)
}
*/

#[allow(clippy::cognitive_complexity)]
pub fn gen_proj(proj: &dir_info::DirInfo, consts: &consts::Consts) -> Result<(), Error> {
    info!("Working on project: {:?}\n", &proj);

    // if skip_templates && proj.proj_dir == "template"
    if proj.proj_dir == "template" {
        return Ok(());
    }

    copy_files_except_tmp(
        &proj.fulldir_str(),
        &format!("{}/tmp/original", &proj.fulldir_str()),
    )
    .with_context(wfh!(
        "Error when copying files into {}/tmp/dir folder.",
        &proj.fulldir_str(),
    ))?;

    dbg!(&proj.info.resources);
    if let Some(ref ress) = &proj.info.resources {
        for res in ress {
            if let Some(ref rule) = res.rule {
                if rule == "front_cover" {
                    if let Some(ref content) = res.content {
                        let origin = &format!(
                            "{}/{}/_asset/_image/{}",
                            &proj.base_dir, &proj.from_dir, content
                        );
                        let dest = &format!("{}/tmp/original/{}", &proj.fulldir_str(), content);
                        dbg!("antes de copiar");
                        fs::copy(&origin, &dest) //
                            .with_context(wfh!(
                                "Error when copying files from {} into {}.",
                                &origin,
                                &dest
                            ))?;
                        dbg!("depois de copiar");
                    }
                }
            }
        }
    }

    let mut authors: Vec<info::InfoPerson2> = vec![];
    dbg!(&proj.info.persons);
    if let Some(ref persons) = &proj.info.persons {
        for p in persons {
            if let Some(ref rule) = &p.rule {
                if rule == "author" {
                    let person = info::InfoPerson2 {
                        name: p.identifier.clone().ok_or_else(|| feh!())?,
                    };
                    authors.push(person);
                }
            }
        }
    }
    dbg!(&authors);

    // lang information
    let all_langs = consts.all_langs.clone();
    let (def_lang, other_langs): (Vec<dir_info::Lang>, Vec<dir_info::Lang>) = all_langs
        .iter()
        .cloned()
        .partition(|lang| lang.to_dir_name == proj.info.translation.language);
    let def_lang: dir_info::Lang = def_lang
        .into_iter()
        .next()
        .ok_or_else(|| format_err!("Failed to get the default language information from preset"))
        .with_context(wfh!())?;

    // TODO: other translations information (to link among themselves)

    for target in proj.info.targets.clone() {
        let dashed_name = format!(
            "{}-{}-{}-{}",
            target.name, target.size, target.orientation, target.reader
        );
        let destination = format!("{}/tmp/{}", &proj.fulldir_str(), dashed_name,);
        copy_files_except_tmp(
            &format!("{}/tmp/original", &proj.fulldir_str()),
            &destination,
        )
        .with_context(wfh!(
            "Error when copying files from {}/tmp/original into {}/tmp/{}.",
            &proj.fulldir_str(),
            &proj.fulldir_str(),
            dashed_name,
        ))?;

        dbg!(&target);
        // TODO: crop and/or resize the cover images, and replace them

        for cover in &target.covers {
            let img_filepath = format!("{}/{}", &destination, cover.cover_file);
            let crop;
            {
                let mut img = image::open(&img_filepath) //
                    .with_context(wfh!(
                        "Error when opening image file from {}.",
                        &img_filepath
                    ))?;
                let (offsetx, offsety) = (cover.cover_dimensions[0], cover.cover_dimensions[1]);
                let (width, height) = img.dimensions();
                let width = if cover.cover_dimensions[2] == 0 {
                    width - offsetx
                } else {
                    cover.cover_dimensions[2]
                };
                let height = if cover.cover_dimensions[3] == 0 {
                    height - offsety
                } else {
                    cover.cover_dimensions[3]
                };
                // TODO: add paper proportion measure, so we can crop exceding width or exceding height
                crop = imageops::crop(&mut img, offsetx, offsety, width, height).to_image();
            }
            crop.save(&img_filepath) //
                .with_context(
                    wfh!(
                        "Error when saving image file to {}.",
                        &img_filepath
                    )
                )?;
        }

        dbg!("");
        println!(
            "Next file is <{}>, for the target <{}>. continue? [Y/n] ",
            &proj.fulldir_str(),
            &target.name
        );

        // setup language change

        let first_lang = proj.info.translation.language.to_string();
        let _sec_langs = proj.info.translation.other_languages.clone();
        for content in proj.info.content_files.iter().map(|c| &c[0]) {
            let path = format!("{}/{}", &destination, &content);

            let mut os = std::fs::read_to_string(&path).with_context(wfh!())?;

            // adds new line around each file
            // so additions may happen in a sole line
            os = format!("\n\n{}\n\n", os);

            let mut ns = String::new();
            let mut file = File::create(&path) //
                .with_context(wfh!("failed to overwrite content file"))?;

            use whatlang::Script;

            let base: whatlang::Lang =
                dir_info::tag_try_into_whatlang(proj.info.translation.language.clone())?;

            let mut whitelist: Vec<whatlang::Lang> = proj
                .info
                .translation
                .other_languages
                .iter()
                .cloned()
                .map(dir_info::tag_try_into_whatlang)
                .collect::<Result<_, _>>()?;
            // pushes the base language
            whitelist.push(base.clone());

            let opt = whatlang::Options::new().set_whitelist(whitelist);

            let mut previous_lang = base;
            let mut previous_script: Option<Script> = None;

            // get all display-math-mode positions
            let maths = os
                .chars()
                .enumerate()
                .filter(|(_i, c)| *c == ':')
                .peekable()
                .fold(
                    (None, 0usize, None, Vec::new()),
                    |(in_math, count, last, mut acc), (i, c)| {
                        match (in_math, count) {
                            // <:>
                            (None, 0usize) => (None, 1, Some(i), acc),
                            // <::> or <: .. :>
                            (None, 1) => {
                                let last = last.unwrap();
                                if last + 1 == i {
                                    // <::>
                                    (None, 2, Some(i), acc)
                                } else {
                                    // <:>
                                    // reseted
                                    (None, 1, Some(i), acc)
                                }
                            }
                            // <:::> or <:: ... :>
                            (None, 2) => {
                                let last = last.unwrap();
                                if last + 1 == i {
                                    // <:::>
                                    (None, 3, Some(i), acc)
                                } else {
                                    // <:: .. :>
                                    (Some(last - 1), 1, Some(i), acc)
                                }
                            }
                            // <::::> or <::: .. :>
                            (None, n) => {
                                let last = last.unwrap();
                                if last + 1 == i {
                                    // <::::>
                                    (None, n + 1, Some(i), acc)
                                } else {
                                    // <::: .. :>
                                    (None, 1, Some(i), acc)
                                }
                            }
                            // <:: ... ::> or <:: ... : .. :>
                            (Some(init_math), 1) => {
                                let last = last.unwrap();
                                if last + 1 == i {
                                    // <:: ... ::>
                                    acc.push((init_math, i));
                                    // reset
                                    (None, 0, None, acc)
                                } else {
                                    // <:: .. : .. :>
                                    (Some(init_math), 1, Some(i), acc)
                                }
                            }
                            (Some(_n), _m) => unreachable!(),
                        }
                    },
                )
                .3;

            // get ponctuation positions, except for ' ponctuation
            let (mut ponctuations, last) = os
                .chars()
                .enumerate()
                .filter(|(_index, c)| !c.is_alphanumeric() && c != &'\'' && !c.is_whitespace())
                .collect::<Vec<_>>()
                .into_iter()
                // the objective is to ignore consecutive ponctuations,
                // in a way that only the first ponctuation is included
                //
                // for this, we reverse them and compare "new" ones (originally earlier)
                // with "last" ones (originally later),
                // and we do not add some of those "last" ones
                // those "last" who are replaced with the "new" ones for the next
                // iteration are ignored from the collection.
                .rev()
                .fold(
                    (Vec::new(), None),
                    |(mut acc, last): (Vec<(_, _)>, Option<(_, _)>), new| {
                        if let Some(last) = last {
                            if let _consecutive @ false = last.0 == new.0 + 1 {
                                acc.push(last);
                            };
                        }
                        (acc, Some(new))
                    },
                );
            // verify the tail, which will be the first (second) ponctuation
            if let Some(last) = last {
                if let Some(old_last) = ponctuations.last().cloned() {
                    if let _consecutive @ false = old_last.0 == last.0 + 1 {
                        ponctuations.push(last)
                    };
                }
            }
            // add a fake first, so the '#' are not messed up
            ponctuations.push((0, '\n'));
            let ponctuations: Vec<_> = ponctuations.into_iter().rev().collect();

            // dbg!("ponctuations:");
            // println!("{:?}", &ponctuations);

            let mut last_pos = 0;
            let mut first_addition = Some(String::new());
            for (index, _ponc) in ponctuations {
                // skip if index is inside a math env
                if let Some((start, _end)) =
                    maths.iter().take_while(|(_start, end)| index < *end).last()
                {
                    if *start < index {
                        continue;
                    }
                }
                let subs: String = os.chars().skip(last_pos).take(index - last_pos).collect();
                let mut subs_to_skip: usize = 0;
                // dbg!(&subs);
                if let Some(detection) = whatlang::detect_with_options(&subs, &opt) {
                    let new_script = detection.script();
                    if detection.is_reliable() || previous_script != Some(new_script) {
                        // this may be unreliable,
                        // but it's safer to change the language when the
                        // script changes
                        let new_lang = detection.lang();

                        if previous_lang == new_lang && previous_script == Some(new_script) {
                            // ignore.. same language and also same script..
                            previous_script = Some(new_script);
                            continue;
                        }
                        // or the language has changed or the script has changed
                        let lang_name = {
                            match new_lang.eng_name().to_lowercase().as_str() {
                                "portuguese" => {
                                    match first_lang.as_str() {
                                        "pt-BR" => {
                                            // instead of "portuguese" being inserted
                                            // in the selectlang babel function..
                                            "brazil"
                                        }
                                        todo => {
                                            // if it is really portuguese, need to insert "portuguese" itself
                                            dbg!(todo);
                                            panic!("Need to implement other language cases");
                                        }
                                    }
                                }
                                "english" => {
                                    match first_lang.as_str() {
                                        "en" => "english",
                                        "pt-BR" => "english",
                                        todo => {
                                            // if it is really portuguese, need to insert "portuguese" itself
                                            dbg!(todo);
                                            panic!("Need to implement other language cases");
                                        }
                                    }
                                }
                                todo => {
                                    dbg!(todo);
                                    panic!("Need to implement other language cases");
                                }
                            }
                        };
                        use info::TargetEngine;
                        let to_append = match target.engine {
                            TargetEngine::Latex => {
                                // two backslashes are used so they can become a single backslash in the future
                                format!("\\\\selectlanguage{{{}}}", lang_name)
                            }
                            TargetEngine::Xelatex => {
                                // TODO
                                dbg!("");
                                panic!("TODO")
                            }
                        };
                        // before adding subs into ns, we better add
                        // the first letters from subs if they are
                        // ponctuations or whitespaces
                        let (indexes, chars): (Vec<usize>, String) = subs
                            .chars()
                            .enumerate()
                            .take_while(|&(_, c): &(_, char)| !c.is_alphanumeric() && c != '\'')
                            .unzip();
                        ns += &chars;
                        subs_to_skip += if let Some(last_index) = indexes.iter().last() {
                            last_index + 1
                        } else {
                            0
                        };

                        previous_lang = new_lang;

                        // only add the "first_addition" after the language has been set
                        if let Some(ref acc_string) = &first_addition {
                            dbg!(&acc_string);
                            dbg!(&ns);
                            dbg!(subs_to_skip);

                            // first set the language
                            ns = to_append.to_string()
                                    // then append the accumulated string which appeared before
                                    // any punctuation
                                    + &acc_string
                                    // then add the text that would have been "already" appended
                                    + &ns;
                        // ns += &to_append;
                        // ns += &acc_string;
                        } else {
                            ns += &to_append;
                        };
                        first_addition = None;
                        previous_script = Some(new_script);
                    } else {
                        // ignore.. since no (easy) detection were reliable
                        // nor any script change were detected
                    }
                } else {
                    // no language information detected
                    // (only ponctuations, empty, etc)
                    // so skip any change..
                }

                // skip chars that were already included
                // some will be skipped only if the language has changed
                let subs: String = subs.chars().skip(subs_to_skip).collect();

                // avoid adding to the real string (ns) if the language has not been
                // detected yet
                if let Some(ref mut acc_string) = first_addition {
                    *acc_string += &subs;
                } else {
                    // only add after it has been assured that some language has been detected
                    // also also added to the string
                    ns += &subs;
                }
                //
                last_pos = index;
            }
            let subs = &os.chars().skip(last_pos).collect::<String>();
            //
            ns += subs;

            // TODO: split text according to non-alphanumerics;

            file.write_all(ns.as_bytes()) //
                .with_context(wfh!(
                    "failed to write on content file that was replaced by regex.",
                ))?;
        }

        let mut initial = match target.name {
            info::TargetName::Article => false,
            info::TargetName::Book => true,
        };

        let mut skip_initial = false;
        let mut sec_active = vec![false; 10];

        // let initial_rank =
        // "ABCDEFGHIJKLMNOPQRSTUVWZ" // ZallmanI
        // "ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÑÒÔÕÖŒÙÚÛÜ" // AM Intex

        // initial
        let mut used_initials = vec![];
        let initials = consts
            .initials
            .iter()
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

            let mut file = File::open(&path) //
                .with_context(wfh!(
                    "failed to open content file to replace by regex. Path: <{}>",
                    &path
                ))?;
            let mut s = String::new();
            file.read_to_string(&mut s) //
                .with_context(
                    wfh!(
                        "failed to read content file to replace by regex.",
                    )
                )?;
            file = File::create(&path) //
                .with_context(wfh!(
                    "failed to overwrite content file to replace by regex.",
                ))?;
            // let mut s2: String = "".into();

            s = format!("\n{}\n", s); // adds new line around each file
                                      // so headers on top of files won't break

            //s = RE_TEST_A.replace_all(&s, "b").to_string(); // test

            s = consts::RE_SYMB_BSLASH
                .replace_all(&s, "\\textbackslash ")
                .to_string();
            // return 2 back slashes into a single real (command) backslash
            s = consts::RE_SYMB_BSLASH2.replace_all(&s, "\\").to_string();
            if proj.info.translation.language.to_string() == "tr" {
                s = consts::RE_SYMB_FI.replace_all(&s, "f\\/i").to_string();
                s = consts::RE_CHAR_MINOR_I_DOTTED
                    .replace_all(&s, "i̇")
                    .to_string(); // TODO: not all is required, just chap names and opening words
                s = consts::RE_CHAR_DOT_DOT.replace_all(&s, "̇").to_string();
            }
            // s = RE_SYMB_CURLY_BRACK.replace_all(&s, "\\{").to_string(); // TODO
            // s = RE_SYMB_CURLY_BRACK2.replace_all(&s, "\\}").to_string(); // TODO
            // TODO underline...
            s = consts::RE_SYMB_AMPER.replace_all(&s, "\\&{}").to_string();
            s = consts::RE_SYMB_AMPER_ESCAPED
                .replace_all(&s, "&")
                .to_string();
            s = consts::RE_SYMB_DOLLAR.replace_all(&s, "\\${}").to_string();
            s = consts::RE_SYMB_PERCENT.replace_all(&s, "\\%{}").to_string();
            // s = RE_SUB_HASH_SPACE_HASH.replace_all(&s, "##").to_string(); // # # -> ## (crowdin messed this up)
            if target.has_parts {
                dbg!("start to test part!");
                // s = RE_SUB_HASH_DOWNGRADE.replace_all(&s, "##").to_string();
                // pub static ref RE_SUB_HASH_DOWNGRADE: Regex = Regex::new("^#(#*)([^#]*)$");
                s = s
                    .lines()
                    .map(|line| {
                        let caps = consts::RE_SUB_HASH_DOWNGRADE.captures(&line);
                        if caps.is_none() {
                            return line.to_string() + "\n";
                        }
                        let caps = caps.unwrap_or_else(|| panic!("{}", &fh!()));
                        if caps.get(0).is_none() {
                            return line.to_string() + "\n";
                        }
                        let c1 = caps.get(1).map_or("", |c| c.as_str());
                        let c2 = caps.get(2).map_or("", |c| c.as_str());
                        if c1.chars().count() > 0 {
                            format!("{}{}", c1, c2) + "\n"
                        } else {
                            dbg!("found a part!");
                            if target.clear_page_active {
                                box_clear_foot[index - 1].0 = true;
                                box_clear_foot[index - 1].1 = true;
                                for a in sec_active.iter_mut().take(9) {
                                    *a = false;
                                }
                            }
                            if target.reset_footer_active {
                                box_clear_foot[index - 1].2 = true;
                            }
                            format!("\\part{{{}}}", c2) + "\n"
                        }
                    })
                    .collect::<String>();
                dbg!("finished to test part!");
            }
            s = consts::RE_SYMB_HASH
                .replace_all(&s, "$1\\texthash{}")
                .to_string();
            s = consts::RE_SYMB_CII
                .replace_all(&s, "$1\\textasciicircum{}")
                .to_string();
            s = consts::RE_SYMB_CII_ESCAPED.replace_all(&s, "^").to_string();
            s = consts::RE_SYMB_DOT_4.replace_all(&s, "    ").to_string();
            s = consts::RE_SYMB_TILDE
                .replace_all(&s, "\\textasciitilde{}")
                .to_string();
            s = consts::RE_CHAR_CJK_COLON.replace_all(&s, "$1:").to_string();
            s = consts::RE_SYMB_COLON_2
                .replace_all(&s, "\n$$$ $1 $$$\n")
                .to_string(); // $1$ ::X::
            s = consts::RE_SYMB_COLON_2_INLINE
                .replace_all(&s, "$$ $1 $$")
                .to_string();
            // TODO: normalize previous replacements inside math-mode

            #[allow(clippy::single_char_pattern)]
            let mut do_initial = |line: &str, start: &str, inis: &mut Vec<char>| {
                // dbg!(&line);
                if line.starts_with(start) {
                    initial = true;
                    skip_initial = false;
                    line.to_string()
                } else if line.starts_with("#") {
                    skip_initial = true;
                    line.to_string()
                } else if initial && !skip_initial {
                    if line.trim() == "" || line.starts_with("[^") {
                        line.to_string()
                    } else {
                        initial = false;
                        let initials: String = line
                            .chars()
                            .take_while(|c| {
                                c.is_alphanumeric() && !c.is_numeric() && !c.is_whitespace()
                            })
                            .collect();
                        let line_start_start: String = initials.chars().take(1).collect();
                        let line_start_end: String = initials.chars().skip(1).collect();
                        let line_start =
                            format!("\\DECORATE{{{}}}{{{}}}", line_start_start, line_start_end);
                        let line_end: String =
                            line.chars().skip(initials.chars().count()).collect();
                        dbg!(&line_start_start);
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

            let mut do_section_clear = |line: &str| {
                dbg!(&line);
                let depth = line.chars().take_while(|&c| c == '#').count();
                if depth == 0 {
                    // line.to_string()
                } else {
                    for a in sec_active.iter_mut().skip(depth).take(9) {
                        *a = false;
                    }
                    if sec_active[depth - 1] {
                        // let line_start: String = "".into();
                        if target.clear_page_active && depth <= target.clear_page_depth as usize {
                            if target.has_parts && depth - 1 == 0 {
                                box_clear_foot[index - 1].0 = true;
                            // line_start += "\\utfbox";
                            } else {
                                box_clear_foot[index - 1].0 = true;
                                box_clear_foot[index - 1].1 = true;
                                // line_start += "\\utfbox\\clearpage";
                            }
                        };
                        if target.reset_footer_active && depth <= target.reset_footer_depth as usize
                        {
                            box_clear_foot[index - 1].2 = true;
                            // line_start += "\\endfoot";
                        }
                    // format!("{}\n\n{}", &line_start, line.to_string())
                    } else {
                        sec_active[depth - 1] = true;
                        // line.to_string()
                    }
                }
            };
            dbg!("finished endfoot and endsec insertions");

            match target.name {
                info::TargetName::Article => {
                    s = s
                        .lines()
                        .map(|line| do_initial(&line, &"# ", &mut used_initials) + "\n")
                        .collect::<String>();
                }
                info::TargetName::Book => {
                    s = s
                        .lines()
                        .map(|line| do_initial(&line, &"# ", &mut used_initials) + "\n")
                        .collect::<String>();
                }
            }

            // section clearing (new page, reset footer)
            if target.reset_footer_active || target.clear_page_active {
                for line in s.lines() {
                    do_section_clear(&line);
                }
            }

            s = consts::RE_PATT_FOOT_ZERO
                .replace_all(&s, "\\blfootnote{$1}\n")
                .to_string(); //
            s = consts::RE_PATT_FOOT_ANON
                .replace_all(&s, "\\blfootnote{$1}\n")
                .to_string(); //
            s = consts::RE_PATT_FOOT_CHAR
                .replace_all(&s, "\\trfootnote{$1}\n")
                .to_string(); //

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

            file.write_all(s.as_bytes()) //
                .with_context(wfh!(
                    "failed to write on content file that was replaced by regex.",
                ))?;
        }

        // repalce every file to add footnote reset, the end of chapter box and clearpage information
        for (content, box_clear_foot) in proj
            .info
            .content_files
            .iter()
            .map(|c| &c[0])
            .zip(box_clear_foot.iter())
        {
            let path = format!("{}/{}", &destination, &content);

            let mut file = File::open(&path) //
                .with_context(wfh!(
                    "failed to open content file to replace by regex. Path: <{}>",
                    &path
                ))?;
            let mut s = String::new();
            file.read_to_string(&mut s) //
                .with_context(
                    wfh!(
                        "failed to read content file to replace by regex."
                    )
                )?;
            // TODO: check if needed
            // s.trim();

            let box_s = if box_clear_foot.0 { "\\utfbox" } else { "" };
            let clear_s = if box_clear_foot.1 { "\\clearpage" } else { "" };
            let foot_s = if box_clear_foot.2 { "\\endfoot" } else { "" };

            file = File::create(&path) //
                .with_context(wfh!(
                    "failed to overwrite content file to replace by regex.",
                ))?;
            file.write_all(format!("\n{}{}{}{}\n", s.trim(), foot_s, box_s, clear_s).as_bytes())
                .with_context(wfh!(
                    "failed to write on content file that was replaced by regex.",
                ))?;
        }
        dbg!("finished the whole substitutions");

        let used_initials_hs: HashSet<char> = HashSet::from_iter(used_initials);
        let sent_initial = if let Some(pos) = initials
            .iter()
            .position(|best| best.is_superset(&used_initials_hs))
        {
            &consts.initials[pos][0]
        } else {
            ""
        };

        // let authors = proj.info.persons_id.iter().
        let info2 = info::Info2 {
            authors: authors.clone(),
            translators: vec![],
            collaborators: vec![],
            thanks: vec![],
            reviewers: vec![],
        };
        let def = dir_info::Defaults {
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

        dbg!("start rendering");

        let mut rendered = consts::TERA
            .render(&format!("{}/main.tex", target.engine), &def)
            .unwrap_or_else(|_| panic!("{}", &fh!()));
        // .map_err(|_| format_err!("Failed to render the tex templates"))
        // .context(fh!())?;

        rendered = consts::RE_FORWARD_ARROW
            .replace_all(&rendered, "{")
            .to_string(); // }

        dbg!("finished rendering");

        let mut mdok = File::create(format!(
            "{}/tmp/{}/main_ok_{}.tex",
            &proj.fulldir_str(),
            dashed_name,
            target.engine
        ))
        .with_context(wfh!("Falied to create latex file"))?;

        mdok.write_fmt(format_args!("{}", rendered))
            .with_context(wfh!("Failed to write on {} tex file", target.engine))?;

        dbg!("TeX file written.");
        dbg!(&target.engine);

        let cdpath = fs::canonicalize(format!(
            "{proj}/tmp/{tgt}",
            proj = &proj.fulldir_str(),
            tgt = &dashed_name
        ))
        .with_context(wfh!(
            "Failed to canonicalize the working project directory."
        ))?
        .into_os_string()
        .into_string()
        .map_err(|e| format_err!("Invalid working directory string path. Error: {:?}", e))
        .with_context(wfh!())?;

        let cmd = match target.engine {
            info::TargetEngine::Latex => format!(
                r#"cd "{cd}" && pdflatex -halt-on-error --shell-escape main_ok_latex.tex "#,
                cd = &cdpath
            ),
            info::TargetEngine::Xelatex => format!(
                r#"cd "{cd}" && xelatex -halt-on-error --shell-escape main_ok_xelatex.tex "#,
                cd = &cdpath
            ),
        };

        dbg!(&cmd);

        for i in 0..consts.passages {
            dbg!(i);
            let output = Command::new("sh")
                .args(&["-c", &cmd])
                .output()
                .with_context(wfh!())?;
            // .context(fh!("Falied to create pdf file"))?;

            if !output.status.success() {
                let err_msg = fh!(
                    "status: {}\n; stdout: {}\n; stderr: {}\n",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );

                dbg!("error when executing xelatex");
                bail!("Error: {}", err_msg);

            // bail!("Error {}.", err_msg);
            // Err(format!("error.. "));
            } else {
                // success
                // copy to output folder

                // output/pt-BR/EEPP/EEPP-pc.pdf
                // const  lang  name name-target.ext

                if i != consts.passages - 1 {
                    continue;
                }

                dbg!("preparing to copy a file..");

                let extension =
                    Path::new(&format!("{}/main_ok_{}.pdf", &destination, target.engine))
                        .extension()
                        .ok_or_else(|| feh!())?
                        .to_string_lossy()
                        .to_string();

                let capitals = proj
                    .proj_dir
                    .chars()
                    .filter(|c| c.is_uppercase())
                    .collect::<String>();
                let out_dest_dir = format!(
                    "{}/{}/{}",
                    consts.output_dir, &def.def_lang.title, &capitals
                );
                let out_dest_file = format!("{}-{}.{}", &proj.proj_dir, dashed_name, extension);
                let out_dest = format!("{}/{}", out_dest_dir, out_dest_file);

                fs::create_dir_all(&out_dest_dir)
                    .with_context(wfh!("Error when creating directories {}", &out_dest_dir))?;

                fs::copy(
                    &format!("{}/main_ok_{}.pdf", &destination, target.engine),
                    out_dest.clone(),
                )
                .with_context(wfh!(
                    "Error when copying files from {}/main_ok_{}.pdf into {}.",
                    &destination,
                    target.engine,
                    &out_dest
                ))?;

                dbg!("file copied to:");
                dbg!(&out_dest);
            }
        }
    }

    Ok(())
}
