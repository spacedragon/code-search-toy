extern crate phf_codegen;

use codegen::Scope;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tree_sitter::Language;

extern "C" {
    fn tree_sitter_javascript() -> Language;
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
    fn tree_sitter_java() -> Language;
}

pub fn sanitize_string(name: &str, escape: bool) -> String {
    let mut result = String::with_capacity(name.len());
    if escape {
        for c in name.chars() {
            match c {
                '\"' => result += "\\\\\\\"",
                '\\' => result += "\\\\\\\\",
                '\t' => result += "\\\\t",
                '\n' => result += "\\\\n",
                '\r' => result += "\\\\r",
                _ => result.push(c),
            }
        }
    } else {
        for c in name.chars() {
            match c {
                '\"' => result += "\\\"",
                '\\' => result += "\\\\",
                '\t' => result += "\\t",
                '\n' => result += "\\n",
                '\r' => result += "\\r",
                _ => result.push(c),
            }
        }
    }
    result
}

pub fn camel_case(name: String) -> String {
    let mut result = String::with_capacity(name.len());
    let mut cap = true;
    for c in name.chars() {
        if c == '_' {
            cap = true;
        } else if cap {
            result.extend(c.to_uppercase().collect::<Vec<char>>());
            cap = false;
        } else {
            result.push(c);
        }
    }
    result
}

pub fn sanitize_identifier(name: &str) -> String {
    if name == "ï»¿" {
        return "BOM".to_string();
    }
    if name == "_" {
        return "UNDERSCORE".to_string();
    }
    if name == "self" {
        return "Zelf".to_string();
    }
    if name == "Self" {
        return "SELF".to_string();
    }

    let mut result = String::with_capacity(name.len());
    for c in name.chars() {
        if ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') || ('0' <= c && c <= '9') || c == '_' {
            result.push(c);
        } else {
            let replacement = match c {
                '~' => "TILDE",
                '`' => "BQUOTE",
                '!' => "BANG",
                '@' => "AT",
                '#' => "HASH",
                '$' => "DOLLAR",
                '%' => "PERCENT",
                '^' => "CARET",
                '&' => "AMP",
                '*' => "STAR",
                '(' => "LPAREN",
                ')' => "RPAREN",
                '-' => "DASH",
                '+' => "PLUS",
                '=' => "EQ",
                '{' => "LBRACE",
                '}' => "RBRACE",
                '[' => "LBRACK",
                ']' => "RBRACK",
                '\\' => "BSLASH",
                '|' => "PIPE",
                ':' => "COLON",
                ';' => "SEMI",
                '"' => "DQUOTE",
                '\'' => "SQUOTE",
                '<' => "LT",
                '>' => "GT",
                ',' => "COMMA",
                '.' => "DOT",
                '?' => "QMARK",
                '/' => "SLASH",
                '\n' => "LF",
                '\r' => "CR",
                '\t' => "TAB",
                _ => continue,
            };
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
            result += replacement;
        }
    }
    result
}

fn cwd() -> PathBuf {
    PathBuf::from(r"..").canonicalize().unwrap()
}

fn gen_for_lang(lang_name: &str, language: &Language) {
    let file_name = format!("language_{}.rs", lang_name.to_lowercase());
    let file = File::create(cwd().join("src/languages").join(file_name)).unwrap();
    let mut file = BufWriter::new(file);

    let names = get_tokens(language);

    let c_name = camel_case(lang_name.to_string());
    gen_enums(lang_name, &mut file, &names);
    gen_map(lang_name, &mut file, &names, c_name);
    gen_enums_str(lang_name, &mut file, &names);
    gen_str_enum(lang_name, &mut file);
    gen_u16_enum(lang_name, &mut file);
    gen_eq(lang_name, &mut file);
}

fn gen_eq(lang_name: &str, file: &mut BufWriter<File>) {
    write!(
        file,
        r###"impl PartialEq<u16> for {} {{
    #[inline(always)]
    fn eq(&self, x: &u16) -> bool {{
        *self == {}::from(*x)
    }}
}}
impl PartialEq<{}> for u16 {{
    #[inline(always)]
    fn eq(&self, x: &{}) -> bool {{
        *x == *self
    }}
}}"###,
        lang_name, lang_name, lang_name, lang_name
    )
    .unwrap();
}

fn gen_str_enum(lang_name: &str, file: &mut BufWriter<File>) {
    /* impl From<&str> for Tsx {
        #[inline(always)]
        fn from(key: &str) -> Self {
            KEYS.get(key).unwrap().clone()
        }
    }*/
    let mut scope = Scope::new();
    scope
        .new_impl(lang_name)
        .impl_trait("From<&str>")
        .new_fn("from")
        .attr("inline(always)")
        .arg("key", "&str")
        .ret("Self")
        .line(format!(
            "{}.get(key).unwrap().clone()",
            lang_name.to_uppercase()
        ));
    write!(file, "{}\n", scope.to_string()).unwrap();
}

fn gen_u16_enum(lang_name: &str, file: &mut BufWriter<File>) {
    /* impl From<u16> for Tsx {
            #[inline(always)]
            fn from(x: u16) -> Self {
                unsafe { std::mem::transmute(x) }
            }
        }
    }*/
    let mut scope = Scope::new();
    scope
        .new_impl(lang_name)
        .impl_trait("From<u16>")
        .new_fn("from")
        .attr("inline(always)")
        .arg("value", "u16")
        .ret("Self")
        .line(format!("unsafe {{ std::mem::transmute(value) }}"));
    write!(file, "{}\n", scope.to_string()).unwrap();
}

fn gen_enums_str(
    lang_name: &str,
    file: &mut BufWriter<File>,
    names: &Vec<(String, bool, String, u16)>,
) {
    let mut scope = Scope::new();
    let impls = scope.new_impl(lang_name);
    impls.impl_trait("Into<&'static str>");
    let newfn = impls.new_fn("into");
    newfn.arg_self().ret("&'static str").line("match self {");
    for (name, _, ts_name, _) in names {
        newfn.line(format!("{}::{} => \"{}\",", lang_name, name, ts_name));
    }
    newfn.line("}");
    write!(file, "{}\n", scope.to_string()).unwrap();
}

fn gen_map(
    lang_name: &str,
    file: &mut BufWriter<File>,
    names: &Vec<(String, bool, String, u16)>,
    c_name: String,
) {
    let mut builder = phf_codegen::Map::new();
    write!(
        file,
        "static {}: phf::Map<&'static str, {}> = ",
        lang_name.to_uppercase(),
        lang_name
    )
    .unwrap();
    for (name, dup, ts_name, _) in names {
        if !dup {
            builder.entry(ts_name.as_str(), format!("{}::{}", c_name, name).as_str());
        }
    }
    write!(file, "{};\n", builder.build()).unwrap();
}

fn gen_enums(
    lang_name: &str,
    file: &mut BufWriter<File>,
    names: &Vec<(String, bool, String, u16)>,
) {
    write!(file, "#[derive(Clone, Debug, PartialEq)]\n").unwrap();
    write!(file, "pub enum {} {{ \n", lang_name).unwrap();

    for (name, _, _, id) in names {
        write!(file, "\t {} = {},\n", name, id).unwrap();
    }
    write!(file, "}}\n").unwrap();
}

fn get_tokens(language: &Language) -> Vec<(String, bool, String, u16)> {
    let mut names = BTreeMap::default();
    let mut name_count = HashMap::new();
    let count = language.node_kind_count();
    for anon in vec![false, true] {
        for i in 0..count {
            let anonymous = !language.node_kind_is_named(i as u16);
            if anonymous != anon {
                continue;
            }
            let kind = language.node_kind_for_id(i as u16).unwrap();
            let name = sanitize_identifier(kind);
            let ts_name = sanitize_string(kind, false);
            let name = camel_case(name);
            let e = match name_count.entry(name.clone()) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() += 1;
                    (format!("{}{}", name, e.get()), true, ts_name, i as u16)
                }
                Entry::Vacant(e) => {
                    e.insert(1);
                    (name, false, ts_name, i as u16)
                }
            };
            names.insert(i, e);
        }
    }
    names.values().map(|x| x.clone()).collect()
}

fn main() {
    let lang_list = vec![
        ("Javascript", unsafe { tree_sitter_javascript() }),
        ("Typescript", unsafe { tree_sitter_typescript() }),
        ("TSX", unsafe { tree_sitter_tsx() }),
        ("Java", unsafe { tree_sitter_java() }),
    ];
    let mut scope = Scope::new();
    let langs_enum = scope.new_enum("Langs").vis("pub");

    let file = File::create(cwd().join("src/languages").join("langs.rs")).unwrap();
    let mut file = BufWriter::new(file);

    for (name, language) in lang_list {
        langs_enum.new_variant(name);
        gen_for_lang(name, &language)
    }

    write!(&mut file, "{}", scope.to_string()).unwrap();
}
