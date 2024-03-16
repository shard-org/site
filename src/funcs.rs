
#![allow(unused_imports)]
use hyper::{
    HeaderMap,
    header::HeaderValue,
    body::Incoming,
    Request, Response, StatusCode,
};
use std::borrow::Cow;
use crate::Error;
use once_cell::sync::Lazy;

static HEADER = Lazy<String> = Lazy::new(|| {
    let file = crate::file_nc("files/header.html").unwrap();
    String::from_utf8_lossy(&file).to_string()
});
#

[allow(unused_variables)]
pub async fn time(uri: Cow<'_, str>, headers: HeaderMap<HeaderValue>, body: Incoming,) -> impl crate::IntoResponse {
let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="styles.css">
    <title>./time.sh</title>
</head>
<body>
    {HEADER}
    <div class="text">
        <div class="center"> 
            <h1 style="font-size: 2em;">{time}</h1>
        </div>
    </div>
</body>
"#)
}
#

[allow(unused_variables)]
pub async fn index(uri: Cow<'_, str>, headers: HeaderMap<HeaderValue>, body: Incoming,) -> impl crate::IntoResponse {
format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="styles.css">
    <title>./slb.sh -h</title>
</head>
    <style>
        .text h2 {{
            margin-bottom: 2px;
        }}
        .text p {{
            margin-top: var(--reduce-margin, 0);
        }}
    </style>
<body>
    {HEADER}

    <div class="text">
        <h2>About Me</h2>
        <p>Hello there, I'm Anthony, the host here.</p>
        <p>I interest in a fair few subjects, including: audio, music, programming, chem, electronics; as well as ttrpgs and wargames.</p>

        <p>If you want to contact me, here are a few ways:</p>
        <div class="indent">
            <p>- <r>email:</r> <i>ant@slb.sh</i></p>
            <p>- <r>discord:</r> <i>anthonyslab</i></p>
        </div>
        <p>Oh and by the way, if you've got any cool projects of your own, suggestions for this site, or anything else really,
        please drop it off in one of the places above. Thanks!</p>



        <h2>Things</h2>
        <p>100% js-free, static, and locally sourced stuff this site has to offer:</p>

        <div class="indent">
            <p>O1. <a href="/dice">dice:</a> a dice roller! Now even works with <i>curl</i>!</p>
            <p>02. <a href="/time">time:</a> ... its just datetime</p>
        </div>



        <h2>Shard?</h2>
        <p><a href="https://repo.shardlang.org">Shard!</a></p>
    </div>
</body>
"#)
}
#

[allow(unused_variables)]
pub async fn blog(uri: Cow<'_, str>, headers: HeaderMap<HeaderValue>, body: Incoming,) -> impl crate::IntoResponse {
fn ls_dir(dir_path: &str) -> Result<Box<[Box<str>]>, Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
        let mut path = entry?.path();
        if !path.is_file() { continue; }

        path.set_extension("");

        files.push(path.display().to_string().into_boxed_str());
    }
    Ok(files.into_boxed_slice())
}

let Ok(blogs) = ls_dir("files/blog/").map_err(|e| println!("{}", e)) else {
    return String::from("Failed to index blogs!");
};

let blog_list = String::new();
for blog in blogs.iter() {
    
}

let file = crate::file("files/header.html").await.unwrap();
let header = String::from_utf8_lossy(&file);

format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>./blog.sh -l</title>
    <link rel="stylesheet" type="text/css" href="styles.css">
</head>
<body>
    {header}
</body>
"#)
}
#

[allow(unused_variables)]
pub async fn dice(uri: Cow<'_, str>, headers: HeaderMap<HeaderValue>, body: Incoming,) -> impl crate::IntoResponse {
fn roll_dice(expr: &str) -> String {
    match expr.parse::<ndm::RollSet>() {
        Ok(r) => r.to_string(),
        Err(_) => String::from("Err"),
    }
}

let result = match uri.split_once("?d=") {
    Some((_, dice)) => {
        // plaintext
        if let Some(dice) = dice.strip_prefix('q') {
            return roll_dice(dice);
        }

        let result = roll_dice(dice);
        format!("<p style=\"margin-top: 40px\">Results:</p><h1>{}</h1>", result)
    },
    None => String::new(),
};

format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>./dice.sh</title>
    <link rel="stylesheet" type="text/css" href="styles.css">

    <style>
        input {{
            margin-top: 10px;
            margin-bottom: 0;
        }}
    </style>
</head>
<body>
    {HEADER}
    <div class="text">
        <p>This is a static dice roller!<br>
        Append <b>?d=d6</b> to the url to roll a d6!</p>

        <p><i>should</i> work for most simple math as well</p>

        <p>Prepend a <b>q</b> to the dice expression to return plaintext.<br>
        Useful for scripting and such ;)</p>

        <p>Alternatively you can type in the expression into the box below!</p>
        <div class="center">
            <form action="dice" method="get">
                <input type="text" id="tag-input" name="d">
            </form>

            {result}
        </div>
    </div>
</body>
"#)
}
