use std::fs;
use std::io::{
    Read, Write, BufRead, BufReader, BufWriter
};

type Error = Box<dyn std::error::Error>;

const IMPORTS: &str = 
"
#![allow(unused_imports)]
use hyper::{
    HeaderMap,
    header::HeaderValue,
    body::Bytes,
    Request, Response, StatusCode,
};
use std::borrow::Cow;
use std::collections::HashMap;
use crate::Error;
";

fn main() {
    if let Err(e) = do_thing() {
        println!("\x1b[33;1mBUILD ERR:\x1b[0m {}", e);
        std::process::exit(1)
    }
}

const MAIN_FILE: &str = "src/main.rs";
const FUNCS_FILE: &str = "src/funcs.rs";

const MATCH_START_KEYWORD: &str = "/* ##FUNC_MATCH_START## */";
const MATCH_END_KEYWORD: &str = "/* ##FUNC_MATCH_END## */";

const FUNC_DIR: &str = "files";

fn do_thing() -> Result<(), Error> {
    let files = get_files(FUNC_DIR);
    println!("{:#?}", files);
    
    let mut match_lines = String::new();
    let mut funcs = String::from(IMPORTS);
    
    for file in files {
        let path = file.strip_prefix(FUNC_DIR).unwrap();

        // if ident.contains('_') {
        //     return Err("`_` in func ident not allowed".into());
        // }

        let mut path = path.strip_suffix(".rs").unwrap();
        let ident = path.strip_prefix('/').unwrap().replace('/', "_");

        if ident == "index" {
            path = "/";
        }

        // dont make it a response func if its internal
        if ident == ".internal" {
            let mut file = fs::File::open(Into::<String>::into(file))?;
            file.read_to_string(&mut funcs)?;
            continue;
        }

        match_lines.push_str(&format!("\"{}\" => funcs::{}(uri, args, parts.headers, body).await.into_response(),\n", path, ident));
        funcs.push_str(&format!("#\n\n[allow(unused_variables)]\npub async fn {}(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {{\n", ident));

        let mut file = fs::File::open(Into::<String>::into(file))?;
        file.read_to_string(&mut funcs)?;

        funcs.push_str("}\n");
    }

    let mut funcs_file = fs::File::create(FUNCS_FILE)?;
    funcs_file.write_all(funcs.as_bytes())?;
    drop(funcs_file);


    replace_lines(match_lines, MATCH_START_KEYWORD, MATCH_END_KEYWORD)?;

    Ok(())
}


fn replace_lines(new_lines: String, start: &str, end: &str) -> Result<(), Error> {
    let file = fs::File::open(MAIN_FILE)?;
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

    let file = fs::File::create(MAIN_FILE)?;
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

fn get_files(dir_path: &str) -> Vec<Box<str>> {
    let mut out = Vec::new();

    for entry in walkdir::WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }

        let filename = entry.path().display().to_string();
        if !filename.ends_with(".rs") { continue; }

        out.push(Box::from(filename));
    }
    out
}
