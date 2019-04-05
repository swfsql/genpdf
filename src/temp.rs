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
        .context(fh!("Failed to create a new {} directory.", to))?;

    let dir = Path::new(from);
    let dirs = read_dir(&dir) //
        .context(fh!(
            "Failed to start copying {} contents into the tmp directory.",
            from
        ))?;

    for path in dirs {
        let path = path //
            .context(fh!("Failed to open a file."))?;
        if path.path().ends_with("tmp") {
            continue;
        }
        let dst = Path::new(to).join(path.path().file_name().unwrap());
        let meta = path
            .metadata()
            .context(fh!("Failed to read {} metadata.", path.path().display()))?;
        if meta.is_dir() {
            fs::create_dir(&dst).context(fh!("Failed to create a new {:?} directory.", &dst))?;
        } else {
            let orig = path.path();
            fs::copy(&orig, &dst) //
                .context(fh!("Failed to copy {:?} into {:?} folder.", &orig, &dst))?;
        }
    }
    Ok(())
}

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
                let mut file = File::open(format!("{}/{}", dir.fulldir_str(), md)).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

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
        ph!("foots: <{:?}>", &ret);
        ret
    };
    let diff_pos = count_foots(original)
        .iter()
        .zip(count_foots(proj))
        .enumerate()
        .inspect(|&(index, (&(ref md, ref foots_a), (_, ref foots_b)))| {
            ph!(" {}: [{}]", index, md);
            foots_a
                .iter()
                .zip(foots_b)
                .inspect(|&(num_a, num_b)| {
                    let diff = if num_a != num_b { " ~" } else { "" };
                    ph!("  {} == {}{}", num_a, num_b, diff);
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
    .with_context(|e| {
        fh!(
            "Error when copying files into {}/tmp/dir folder. Due to {}.",
            &proj.fulldir_str(),
            e
        )
    })?;

    ph!("res: <{:?}>", proj.info.resources);
    if let &Some(ref ress) = &proj.info.resources {
        for res in ress {
            if let Some(ref rule) = res.rule {
                if rule == "front_cover" {
                    if let Some(ref content) = res.content {
                        let origin = &format!(
                            "{}/{}/_asset/_image/{}",
                            &proj.base_dir, &proj.from_dir, content
                        );
                        let dest = &format!("{}/tmp/original/{}", &proj.fulldir_str(), content);
                        ph!("antes de copiar");
                        fs::copy(&format!("{}", &origin), &format!("{}", &dest)) //
                            .context(fh!(
                                "Error when copying files from {} into {}.",
                                &origin,
                                &dest
                            ))?;
                        ph!("depois de copiar");
                    }
                }
            }
        }
    }

    let mut authors: Vec<info::InfoPerson2> = vec![];
    ph!("authors: <{:?}>", &proj.info.persons);
    if let &Some(ref persons) = &proj.info.persons {
        for p in persons {
            if let &Some(ref rule) = &p.rule {
                if rule == "author" {
                    let person = info::InfoPerson2 {
                        name: p.identifier.clone().unwrap(),
                    };
                    authors.push(person);
                }
            }
        }
    }
    ph!("authors: <{:?}>", &authors);

    // lang information
    let all_langs = consts.all_langs.clone();
    let (def_lang, other_langs): (Vec<dir_info::Lang>, Vec<dir_info::Lang>) = all_langs
        .iter()
        .cloned()
        .partition(|lang| lang.to_dir_name == proj.info.translation.language);
    let def_lang: dir_info::Lang = def_lang
        .into_iter()
        .next()
        .ok_or(format_err!(
            "Failed to get the default language information from preset"
        ))
        .context(fh!())?;

    // TODO: other translations information (to link among themselves)

    for target in proj.info.targets.clone() {
        let destination = format!("{}/tmp/{}", &proj.fulldir_str(), target.name);
        copy_files_except_tmp(
            &format!("{}/tmp/original", &proj.fulldir_str()),
            &destination,
        )
        .with_context(|e| {
            fh!(
                "Error when copying files from {}/tmp/original into {}/tmp/{}. Due to {}.",
                &proj.fulldir_str(),
                &proj.fulldir_str(),
                target.name,
                e
            )
        })?;

        ph!("target: <{:?}>", target);
        // TODO: crop and/or resize the cover images, and replace them

        for cover in &target.covers {
            let img_filepath = format!("{}/{}", &destination, cover.cover_file);
            let mut crop;
            {
                let mut img = image::open(&img_filepath) //
                    .with_context(|e| {
                        fh!(
                            "Error when opening image file from {}. Due to {}.",
                            &img_filepath,
                            e
                        )
                    })?;
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
                .with_context(|e| {
                    fh!(
                        "Error when saving image file to {}. Due to {}.",
                        &img_filepath,
                        e
                    )
                })?;
        }

        ph!(
            "Next file is <{}>, for the target <{}>. continue? [Y/n] ",
            &proj.fulldir_str(),
            &target.name
        );

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
                .with_context(|e| {
                    fh!(
                        "failed to open content file to replace by regex. Error: {:?}. Path: <{}>",
                        e,
                        &path
                    )
                })?;
            let mut s = String::new();
            file.read_to_string(&mut s) //
                .with_context(|e| {
                    fh!(
                        "failed to read content file to replace by regex. Error: {:?}",
                        e
                    )
                })?;
            file = File::create(&path) //
                .with_context(|e| {
                    fh!(
                        "failed to overwrite content file to replace by regex. Error: {:?}",
                        e
                    )
                })?;
            // let mut s2: String = "".into();

            s = format!("\n{}\n", s); // adds new line around each file
                                      // so headers on top of files won't break

            //s = RE_TEST_A.replace_all(&s, "b").to_string(); // test

            s = consts::RE_SYMB_BSLASH
                .replace_all(&s, "\\textbackslash ")
                .to_string();
            if proj.info.translation.language == "tr" {
                s = consts::RE_SYMB_FI.replace_all(&s, "f\\/i").to_string();
                s = consts::RE_CHAR_i_DOTTED.replace_all(&s, "i̇").to_string(); // TODO: not all is required, just chap names and opening words
                s = consts::RE_CHAR_DOT_DOT.replace_all(&s, "̇").to_string();
            }
            // s = RE_SYMB_CURLY_BRACK.replace_all(&s, "\\{").to_string(); // TODO
            // s = RE_SYMB_CURLY_BRACK2.replace_all(&s, "\\}").to_string(); // TODO
            // TODO underline...
            s = consts::RE_SYMB_AMPER.replace_all(&s, "\\&{}").to_string();
            s = consts::RE_SYMB_DOLLAR.replace_all(&s, "\\${}").to_string();
            s = consts::RE_SYMB_PERCENT.replace_all(&s, "\\%{}").to_string();
            // s = RE_SUB_HASH_SPACE_HASH.replace_all(&s, "##").to_string(); // # # -> ## (crowdin messed this up)
            if target.has_parts {
                ph!("start to test part!");
                // s = RE_SUB_HASH_DOWNGRADE.replace_all(&s, "##").to_string();
                // pub static ref RE_SUB_HASH_DOWNGRADE: Regex = Regex::new("^#(#*)([^#]*)$").unwrap();
                s = s
                    .lines()
                    .map(|ref line| {
                        let caps = consts::RE_SUB_HASH_DOWNGRADE.captures(&line);
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
                            ph!("found a part!");
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
                    })
                    .collect::<String>();
                ph!("finished to test part!");
            }
            s = consts::RE_SYMB_HASH
                .replace_all(&s, "$1\\texthash{}")
                .to_string();
            s = consts::RE_SYMB_CII
                .replace_all(&s, "$1\\textasciicircum{}")
                .to_string();
            s = consts::RE_SYMB_DOT_4.replace_all(&s, "    ").to_string();
            s = consts::RE_SYMB_TILDE
                .replace_all(&s, "\\textasciitilde{}")
                .to_string();
            s = consts::RE_CHAR_CJK_COLON.replace_all(&s, "$1:").to_string();
            s = consts::RE_SYMB_COLON_2
                .replace_all(&s, "$$$ $1 $$$")
                .to_string(); // $1$ ::X::
            s = consts::RE_SYMB_COLON_2_INLINE
                .replace_all(&s, "$$ $1 $$")
                .to_string();
            // TODO: normalize previous replacements inside math-mode

            let mut do_section_clear = |line: &str| {
                ph!("Debug line:\n{}", &line);
                let depth = line.chars().take_while(|&c| c == '#').count();
                if depth == 0 {
                    // line.to_string()
                } else {
                    for i in depth..9 {
                        sec_active[i] = false;
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
                ()
            };
            ph!("finished endfoot and endsec insertions");

            let mut do_initial = |line: &str, start: &str, inis: &mut Vec<char>| {
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
                        ph!("line start start: <{:?}>", &line_start_start);
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
                s.lines()
                    .map(|line| do_section_clear(&line))
                    .collect::<()>();
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
                .with_context(|e| {
                    fh!(
                        "failed to write on content file that was replaced by regex. Error: {:?}",
                        e
                    )
                })?;
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
                .with_context(|e| {
                    fh!(
                        "failed to open content file to replace by regex. Error: {:?}. Path: <{}>",
                        e,
                        &path
                    )
                })?;
            let mut s = String::new();
            file.read_to_string(&mut s) //
                .with_context(|e| {
                    fh!(
                        "failed to read content file to replace by regex. Error: {:?}",
                        e
                    )
                })?;
            s.trim();

            let box_s = if box_clear_foot.0 { "\\utfbox" } else { "" };
            let clear_s = if box_clear_foot.1 { "\\clearpage" } else { "" };
            let foot_s = if box_clear_foot.2 { "\\endfoot" } else { "" };

            file = File::create(&path) //
                .with_context(|e| {
                    fh!(
                        "failed to overwrite content file to replace by regex. Error: {:?}",
                        e
                    )
                })?;
            file.write_all(format!("\n{}{}{}{}\n", s.trim(), foot_s, box_s, clear_s).as_bytes())
                .with_context(|e| {
                    fh!(
                        "failed to write on content file that was replaced by regex. Error: {:?}",
                        e
                    )
                })?;
        }
        ph!("finished the whole substitutions");

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

        ph!("start rendering");

        let mut rendered = consts::TERA
            .render("main.tex", &def)
            // .map_err(|_| format_err!("Failed to render the tex templates"))
            .unwrap();
        // .context(fh!())?;

        rendered = consts::RE_FORWARD_ARROW
            .replace_all(&rendered, "{")
            .to_string(); // }
        debug!("{}", rendered);

        ph!("finished rendering");

        let mut mdok = File::create(format!(
            "{}/tmp/{}/main_ok.tex",
            &proj.fulldir_str(),
            target.name
        ))
        .context(fh!("Falied to create tex file"))?;

        mdok.write_fmt(format_args!("{}", rendered))
            .context(fh!("Failed to write on tex file"))?;

        info!("TeX file written.");

        let cdpath = fs::canonicalize(format!(
            "{proj}/tmp/{tgt}",
            proj = &proj.fulldir_str(),
            tgt = &target.name
        ))
        .context(fh!("Failed to canonicalize the working project directory."))?
        .into_os_string()
        .into_string()
        .map_err(|e| format_err!("Invalid working directory string path. Error: {:?}", e))
        .context(fh!())?;

        //let cmd = format!("xelatex main_ok.tex -include-directory=\"{cd}\" -output-directory=\"{cd}\" -halt-on-error --shell-escape",
        //let cmd = format!("xelatex \"{cd}\\main_ok.tex\" -halt-on-error --shell-escape",
        //let cmd = format!("\"cd /d \"{cd}\" && xelatex main_ok.tex -halt-on-error --shell-escape\"",
        //let cmd = format!("cd ../transifex && ls");

        let cmd = &format!(
            r#"cd "{cd}" && xelatex -halt-on-error --shell-escape main_ok.tex "#,
            //let cmd = OsStr::new(&cmd);
            cd = &cdpath
        );
        //cd=&proj.dir[2..]);
        ph!("Command:\n{:?}", &cmd);

        //xelatex main_ok.tex -include-directory="C:/Users/Thiago/Desktop/ancap.ch/transifex/from_th/the essay name/tmp/book" -output-directory="C:/Users/Thiago/Desktop/ancap.ch/transifex/from_th/the essay name/tmp/book" -halt-on-error --shell-escape

        for i in 0..consts.passages {
            ph!("passage {}", i);
            let output = Command::new("sh").args(&["-c", cmd]).output().unwrap();
            // .context(fh!("Falied to create pdf file"))?;

            if !output.status.success() {
                let err_msg = fh!(
                    "status: {}\n; stdout: {}\n; stderr: {}\n",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );

                ph!("error when executing xelatex: \n{}", err_msg);

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

                ph!("preparing to copy a file..");

                let extension = Path::new(&format!("{}/main_ok.pdf", &destination))
                    .extension()
                    .unwrap()
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
                let out_dest_file = format!("{}-{}.{}", &proj.proj_dir, target.name, extension);
                let out_dest = format!("{}/{}", out_dest_dir, out_dest_file);

                fs::create_dir_all(&out_dest_dir)
                    .context(fh!("Error when creating directories {}", &out_dest_dir))?;

                fs::copy(
                    &format!("{}/main_ok.pdf", &destination),
                    format!("{}", &out_dest),
                )
                .context(fh!(
                    "Error when copying files from {}/main_ok.pdf into {}.",
                    &destination,
                    &out_dest
                ))?;

                ph!("\n->file copied to: \n{}\n", &out_dest);
            }
        }
    }

    Ok(())
}
