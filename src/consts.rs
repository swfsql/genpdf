
use crate::dir_info::Lang;
use crate::VS;
use regex::Regex;
use tera::Tera;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Consts {
    pub min_ver: String,
    pub passages: u8,
    pub cover_nodes: Vec<String>,
    pub all_langs_from_dir: String,
    pub all_langs_to_dir: String,
    pub output_dir: String,
    pub initials: Vec<VS>,
    pub num_cpu: u8,
    pub all_langs: Vec<Lang>,
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
