use std::fs;
use std::io::{
    Write, BufRead, BufReader, BufWriter
};
use std::fmt::Write as FmtWrite;
use std::collections::HashMap;

const IMPORTS: &str = "#![allow(unused_imports)] use crate::*;use crate::funcs::*;\n";

const FUNCS_DIR: &str = "src/funcs/";
const PROJECT_DIR: &str = "files";

enum Method {
    Get,
    Post,
}

use std::fmt;
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Method::*;
        let out = match self {
            Get => "Get",
            Post => "Post",
        };
        write!(f, "{out}")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = get_files(PROJECT_DIR);

    let mut routes = String::new();
    let mut funcs: HashMap<String, String> = HashMap::new(); //ident, contents

    for file in files {
        let path = file.strip_prefix(PROJECT_DIR).unwrap();

        if let Some(path) = path.strip_suffix(".rs") {
            let ident = path.strip_prefix('/').unwrap();
            let mut path = String::from(path);

            let content = std::fs::read_to_string(file.as_ref())?;

            if ident == ".internal" {
                funcs.insert(String::from("internal"), content);
                continue;
            }

            if ident == "index" {
                path = String::from("/");
            }

            let methods = get_funcs(&content); 

            if let Some(new_path) = path.strip_suffix("_") {
                path = format!("{new_path}/*");
                let path = path.strip_suffix("*").unwrap();
                if !methods.is_empty() {
                    let mut route_str = methods.iter()
                        .fold(format!(".add_route(\"{path}\",("), |mut acc, m| {
                            acc.push_str(&format!("{m}(funcs::{ident}::{}),", m.to_string().to_lowercase())); acc
                        });

                    if route_str.ends_with(",") {
                        route_str.pop();
                    }
                    route_str.push_str("))\n");
                    routes.push_str(&route_str);
                }
            }
            
            if !methods.is_empty() {
                let mut route_str = methods.iter()
                    .fold(format!(".add_route(\"{path}\",("), |mut acc, m| {
                        acc.push_str(&format!("{m}(funcs::{ident}::{}),", m.to_string().to_lowercase())); acc
                    });

                if route_str.ends_with(",") {
                    route_str.pop();
                }
                route_str.push_str("))\n");
                routes.push_str(&route_str);
            }

            funcs.insert(String::from(ident), content);
            continue;
        }

        if let Some(ident) = path.strip_prefix("/_") {
            let (mut ident, _) = ident.split_once('.').unwrap_or((ident, ""));
            
            if ident == "index" {
                ident = "";
            }

            routes.push_str(&format!(".add_route(\"/{ident}\", Get(|c: Query<CacheLen>, f: Query<Files>| get_file(c, f, Url(\"{path}\"))))\n", ))
        }
    }

    replace_lines(routes, "src/main.rs", "/* ##FUNC_START## */", "/* ##FUNC_END## */")?;

    let mut mod_file_lines = String::new();

    for (ident, contents) in funcs {
        let path = format!("{FUNCS_DIR}{ident}.rs");
        let contents = String::from(IMPORTS) + &contents;
        write_to_file(&path, contents)?;
        writeln!(mod_file_lines, "pub mod {ident};")?;
    }

    write_to_file("src/funcs/mod.rs", mod_file_lines)?;

    Ok(())
}

fn get_files(dir_path: &str) -> Vec<Box<str>> {
    let mut out = Vec::new();

    for entry in walkdir::WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }

        let filename = entry.path().display().to_string();
        // if !filename.ends_with(".rs") { continue; }

        out.push(Box::from(filename));
    }
    out
}

fn get_funcs(contents: &str) -> Vec<Method> {
    let mut out = Vec::new();
    let Ok(file) = syn::parse_file(contents) else {
        return out;
    };

    for item in &file.items {
        if let syn::Item::Fn(f) = item {
            use Method::*;
            out.push(match f.sig.ident.to_string().as_str() {
                "get" => Get,
                "post" => Post,
                _ => continue,
            });
        }
    }
    out
}

fn write_to_file(name: &str, contents: String) -> std::io::Result<()> {
    let mut funcs_file = fs::File::create(name)?;
    funcs_file.write_all(contents.as_bytes())?;
    Ok(())
}

fn replace_lines(new_lines: String, name: &str, start: &str, end: &str) -> std::io::Result<()> {
    let file = fs::File::open(name)?;
    let reader = BufReader::new(file);

    let mut lines_before_start = Vec::new();
    let mut lines_after_end = Vec::new();

    let mut lines = reader.lines();

    'main: while let Some(line) = lines.next() {
        let line = line?;

        if line.trim() != start {
            lines_before_start.push(line);
            continue;
        }

        while let Some(line) = lines.next() {
            if line?.trim() == end { 
                lines_after_end = lines
                    .map(|l| l.unwrap())
                    .collect();
                break 'main; 
            }
        }
    }

    let file = fs::File::create(name)?;
    let mut writer = BufWriter::new(file);

    for l in lines_before_start {
        writeln!(writer, "{}", l)?;
    }

    writeln!(writer, "{}\n{}\n{}", start, new_lines, end)?;

    for l in lines_after_end {
        writeln!(writer, "{}", l)?;
    }

    Ok(())
}

