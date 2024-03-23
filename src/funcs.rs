
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
#

[allow(unused_variables)]
pub async fn ftoc(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {
let mut result = String::new();

if let Some(expr) = args.get("c") {
    result = match expr.parse::<i64>() {
        Ok(n) => ((n * 9 / 5) + 32).to_string(),
        Err(e) => e.to_string(),
    };
}

if let Some(expr) = args.get("f") {
    result = match expr.parse::<i64>() {
        Ok(n) => ((n - 32) * 5 / 9).to_string(),
        Err(e) => e.to_string(),
    };
}

if args.contains_key("q") {
    return result;
}

if !result.is_empty() {
    result = format!("<p style=\"margin-top: 40px\">Results:</p><h1>{result}</h1>");
}

format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>./dice.sh</title>
    <link rel="stylesheet" type="text/css" href="styles/styles.css">

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
        <p>Fahrenheit to Celsius converter (or the other way around)<br>
        Use <c>?c=celsius</c> or <c>?f=fahrenheit</c> to convert to the other unit</p>

        <p>Add a <c>&q</c> tag to return plaintext.</p>

        <p>You can also type a value into one of the boxes below:</p>
        <div class="center">
            <form action="ftoc" method="get">
                <input type="text" id="tag-input" name="f" placeholder="F">
            </form>
            <form action="ftoc" method="get">
                <input type="text" id="tag-input" name="c" placeholder="C">
            </form>

            {result}
        </div>
    </div>
</body>
"#)
}
#

[allow(unused_variables)]
pub async fn news(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {
if let Some(token) = args.get("t") {
    let token: &str = &token;
    if !TOKENS.contains(&token) {
        return crate::Res(String::from("Invalid Token.. nice try :p"), StatusCode::UNAUTHORIZED);
    }

    if body.is_empty() {
        return crate::Res(String::from("Empty Body"), StatusCode::BAD_REQUEST);
    }

    let body = String::from_utf8_lossy(&body);
    NEWS.add(&body).await.unwrap();

    return crate::Res(String::new(), StatusCode::OK);
}


let news = NEWS.read().await;
if args.contains_key("q") {
    let news = match args.get("n") {
        Some(n) => {
            let Ok(n) = n.parse::<usize>() else {
                return crate::Res(String::from("Invalid Index"), StatusCode::BAD_REQUEST);
            };

            let Some(news) = news.get(n) else {
                return crate::Res(String::from("Invalid Index"), StatusCode::BAD_REQUEST);
            };
            news
        },
        None => news.first().unwrap(),
    };

    let news_time = Local.timestamp_opt(news.0 as i64, 0)
        .unwrap().format("%d-%m-%y");

    // remove html tags from msg for plaintext display
    let mut skip = false;
    let msg = news.1.chars().fold(String::new(), |mut acc, c| {
        match c {
            '<' => skip = true,
            '>' if skip => skip = false,
            _ if skip => (),
            c => acc.push(c),
        } acc
    });

    return crate::Res(format!("{news_time}: {msg}"), StatusCode::OK);
}

let news_str = news.iter().rev().fold(String::new(), |mut acc, (t, m)| {
    let time = Local.timestamp_opt(*t as i64, 0)
        .unwrap().format("%d-%m-%y");

    acc.push_str(&format!("<p><r>{}:</r> {}</p>", time, m)); acc
});

crate::Res(format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="styles/styles.css">
    <title>./news.sh</title>
</head>
<body>
    {HEADER}
    <div class="text">
        {news_str}
    </div>
</body>
"#), StatusCode::OK)
}
const TOKENS: &[&str] = &[
    "MBrgJ15MF5jinC7KbgmZv7QhvtzRL4Znr7R38PSH5k", // anthony
];



lazy_static::lazy_static! {
    static ref HEADER: String = {
        let file = crate::file_nc("files/header.html").unwrap();
        String::from_utf8_lossy(&file).to_string()
    };

}

use std::fmt;
impl fmt::Display for HEADER {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &**self)
    }
}





use tokio::{
    sync::RwLock,
    io::AsyncWriteExt,
    fs::File,
};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local, TimeZone};

type News = Vec<(u64, Box<str>)>;

const NEWS_FILENAME: &str = "files/.news";

lazy_static::lazy_static! {
    static ref NEWS: RwLock<News> = {
        let mut file = std::fs::File::open(NEWS_FILENAME).unwrap();

        let mut content = String::new();
        let _ = file.read_to_string(&mut content).unwrap();

        NEWS::from(&content)
    };
}

impl NEWS {
    fn from(input: &str) -> RwLock<News> {
        let news = input.lines().fold(Vec::new(), |mut acc, l| {
            let (date, msg) = l.split_once(',')
                .expect("malformed config file");

            let date: u64 = date.parse()
                .expect("invalid date");

            acc.push((date, Box::from(msg))); acc
        });

        RwLock::new(news)
    }

    async fn add(&self, text: &str) -> tokio::io::Result<()> {
        // let time = chrono::Local::now()
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap().as_secs();

        let mut news = self.write().await;
        news.push((time, Box::from(text)));
        drop(news);
        
        self.save().await
    }

    async fn save(&self) -> tokio::io::Result<()> {
        let news = self.read().await;
        let contents = news.iter().fold(String::new(), |mut acc, (t, m)| {
            acc.push_str(&format!("{},{}\n", t, m)); acc
        });
        drop(news);
        
        let mut file = File::create(NEWS_FILENAME).await?;
        file.write_all(contents.as_bytes()).await?;

        Ok(())
    }
}





const MUSICS_FILENAME: &str = "files/.musics";
type Musics = Vec<(Box<str>, Box<str>)>;
lazy_static::lazy_static! {
    static ref MUSICS: RwLock<Musics> = {
        let mut file = std::fs::File::open(MUSICS_FILENAME).unwrap();

        let mut content = String::new();
        let _ = file.read_to_string(&mut content).unwrap();

        MUSICS::from(&content)
    };
}

impl MUSICS {
    fn from(input: &str) -> RwLock<Musics> {
        let news = input.lines().fold(Vec::new(), |mut acc, l| {
            let line = l.split_once(',')
                .expect("malformed config file");

            acc.push((Box::from(line.0), Box::from(line.1))); acc
        });

        RwLock::new(news)
    }
    async fn add(&self, text: &str) -> tokio::io::Result<()> {
        let line = text.split_once(',')
            .expect("malformed config file");

        let mut news = self.write().await;
        news.push((Box::from(line.0), Box::from(line.1)));
        drop(news);
        
        self.save().await
    }

    async fn save(&self) -> tokio::io::Result<()> {
        let news = self.read().await;
        let contents = news.iter().fold(String::new(), |mut acc, (n, l)| {
            acc.push_str(&format!("{n},{l}\n")); acc
        });
        drop(news);
        
        let mut file = File::create(MUSICS_FILENAME).await?;
        file.write_all(contents.as_bytes()).await?;

        Ok(())
    }
}
#

[allow(unused_variables)]
pub async fn time(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {
let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="styles/styles.css">
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
pub async fn index(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {
let news = NEWS.read().await;
let news = news.last().unwrap();

let news_time = Local.timestamp_opt(news.0 as i64, 0)
    .unwrap().format("%d-%m-%y");
let news_msg = &news.1;


format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="styles/styles.css">
    <title>./slb.sh -h</title>

    <style>
        .text p {{
            margin-top: var(--reduce-margin, 0);
        }}
    </style>
</head>
<body>
    {HEADER}

    <div class="text">
        <p><r>{news_time}:</r> {news_msg}</p>

        <h2>About Me</h2>
        <p>Hello there, I'm Anthony, the host here.</p>
        <p>I interest in a fair few subjects, including: audio, music, programming, chem, electronics; as well as ttrpgs and wargames.</p>

        <p>If you want to contact me, here are a few ways:</p>
        <div class="indent">
            <p>- <r>email:</r> <i>ant@slb.sh</i> (not up yet)</p>
            <p>- <r>discord:</r> <i>anthonyslab</i></p>
        </div>
        <p>Oh and by the way, if you've got any cool projects of your own, suggestions for this site, or anything else really,
        please drop it off in one of the places above. Thanks!</p>


        <h2>Projects</h2>
        <div class="indent">
            <p>01. <a href="https://repo.shardlang.org">shard:</a> A low-level programming language, focusing on accurately modeling hardware,
            avoiding abstractions, keeping a terse syntax, and providing a detailed macro system. <a href="https://discord.gg/z3Qnr87e7c"><i>help wanted</i></a></p>
            <p>02. <a href="/rp">rp:</a> This site is actually built with <r>rp</r>! It can generate and serve pages from rust code, for a js-free web!</p>
            <p>03. <a href="/tchat">tchat:</a> Tool for monitoring a twitch chat from the terminal! Supports logging, multiple chats, timestamps and several kinds of badge display; all with no token required.</p>
            <p>04. <a href="https://github.com/anthonysSlab/pot_rust">pot:</a> A discord bot, created specifically for the Wicked Wizard's <a href="https://discord.com/invite/ACuvMvzHjz">discord server</a>. <i>unmaintained</i></a>
        </div>


        <h2>Utils</h2>
        <p>100% js-free, static, and locally sourced mini-utils hosted on here:</p>
        <div class="indent">
            <p>O1. <a href="/dice">dice:</a> a dice roller! Now even works with <i>curl</i>!</p>
            <p>02. <a href="/time">time:</a> ... its just datetime</p>
            <p>03. <a href="/news">news:</a> a news system displaying the latest under the header, and allowing me to add new ones with a single POST request. Add a <c>?q</c> for plaintext output!</p>
            <p>04. <a href="/ftoc">ftoc:</a> Fahrenheit to Celsius converter (or the other way around)</p>
        </div>


    </div>
</body>
"#)
}
#

[allow(unused_variables)]
pub async fn blog(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {
use std::path::{PathBuf, Path};

fn ls_dir(dir_path: &str) -> Result<Box<[PathBuf]>, Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
        let path = entry?.path();
        if !path.is_file() { continue; }

        files.push(path);
    }
    Ok(files.into_boxed_slice())
}

fn creation_time(path: &Path) -> i64 {
    path.metadata().unwrap()
    .created().unwrap()
    .duration_since(UNIX_EPOCH)
    .unwrap().as_secs() as i64
}

let Ok(mut blogs) = ls_dir("files/blog/") else {
    return String::from("Failed to index blogs!");
};

use std::time::UNIX_EPOCH;
use chrono::TimeZone;

blogs.sort_by(|a, b| creation_time(b).cmp(&creation_time(a)));

let mut blog_list = String::new();
for blog in blogs.iter() {
    let filetime = chrono::Local
        .timestamp_opt(creation_time(blog), 0)
        .unwrap().format("%d-%m-%y");

    let filename = blog
        .file_name().unwrap()
        .to_str().unwrap();

    let filename_display = Path::new(filename)
        .with_extension("");
    let filename_display = filename_display
        .display().to_string()
        .replace('_', " ");

    blog_list.push_str(&format!(r#"<h2>{filetime}: <a href="/blog/{filename}">{filename_display}</a></h2>{}"#, "\n"));
}

format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>./blog.sh -l</title>
    <link rel="stylesheet" type="text/css" href="styles/styles.css">
</head>
<body>
    {HEADER}
    <div class="text">
        {blog_list}
    </div>
</body>
"#)
}
#

[allow(unused_variables)]
pub async fn dice(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {
let result = match args.get("d") {
    Some(expr) => {
        let result = match expr.parse::<ndm::RollSet>() {
            Ok(r) => r.to_string(),
            Err(_) => String::from("Err"),
        };

        if args.contains_key("q") {
            return result;
        }

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
    <link rel="stylesheet" type="text/css" href="styles/styles.css">

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
        Use <c>?d=d8</c> to roll a d8! Now also supporting simple math like <c>2d6+3</c>.</p>

        <p>Add a <c>&q</c> tag to return plaintext. Useful for scripting and such ;)</p>

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
#

[allow(unused_variables)]
pub async fn rocks(uri: &str, args: HashMap<&str, String>, headers: HeaderMap<HeaderValue>, body: Bytes) -> impl crate::IntoResponse {
if let Some(token) = args.get("t") {
    let token: &str = &token;
    if !TOKENS.contains(&token) {
        return crate::Res(String::from("Invalid Token.. nice try :p"), StatusCode::UNAUTHORIZED);
    }

    if body.is_empty() {
        return crate::Res(String::from("Empty Body"), StatusCode::BAD_REQUEST);
    }

    let body = String::from_utf8_lossy(&body);
    MUSICS.add(&body).await.unwrap();

    return crate::Res(String::new(), StatusCode::OK);
}

let musics = MUSICS.read().await;

use rand::seq::SliceRandom;
if args.contains_key("r") {
    let (_, link) = musics.choose(&mut rand::thread_rng()).unwrap();
    return crate::Res(link.to_string(), StatusCode::OK);
}

use std::fmt::Write;
let (list, _) = musics.iter().fold((String::new(), 0_u8), |mut acc, (n, l)| {
    acc.1 += 1;
    let _ = writeln!(acc.0, "<p>{:02x}. <a href=\"{l}\">{n}</a></p>", acc.1);
    acc
});

crate::Res(format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="styles/styles.css">
    <title>./rocks.sh -h</title>
    <style>
        .center p {{
            margin-top: 0;
        }}
    </style>
</head>
<body>
    {HEADER}
    <div class=center>
        <p>Stuff that rocks!</p>
    </div>
    <div class=text>
        <h2>Software</h2>
        <div class=indent>
            <p>01. <a href="http://eradman.com/entrproject/">entr:</a> Runs any command whenever a chosen file changes. Useful for recompiling and running a project whenever you update the source.</p>
            <p>02. <a href="https://github.com/rvaiya/warpd">warpd:</a> Keyboard driven virtual pointer. Lets you fully control your mouse with the keyboard!</p>
            <p>03. <a href="https://github.com/tulir/gomuks">gomucks:</a> Terminal based matrix client. Includes image previews and emoji support.</p>
            <p>04. <a href="https://spade-lang.org/">spade:</a> A rusty HDL (hardware description language). Clean and actively maintained.</p>
            <p>05. <a href="https://typst.app/">typst:</a> Markup-based typesetting system, like LaTeX but much cleaner, elegant, and easier to learn.</p>
            <!-- <p>06. <a href=""></a> <p> -->
        </div>

        <h2>Fonts</h2>
        <div class=indent>
            <p>01. <a href="https://fsd.it/shop/fonts/pragmatapro/">PragmataPro:</a> This is my daily driver for all termial windows, 
            it's quite condensed and yet even more readable than fonts like <c>Hack</c> or <c>Fira Code</c>.
            Only drawback is that it's sold for an absolutely exorbitant sum (200€), creator must've been dropped on the head as a child or idk.
            Luckly there's a <i>very</i> similar open source font <a href="https://github.com/shytikov/pragmasevka">Pragmasevka</a>. The latter is used <c>on this website as code blocks</c>.</p>
            <p>02. <a href="https://fonts.adobe.com/fonts/adhesive-nr-seven">Adhesive Nr. Seven</a> Modern Blackletter, created using torn adhesive tape. Great for banners, titles, and posters.</p>
            <p>03. <a href="https://b.agaric.net/page/agave">agave:</a> A very smooth and round font, Looks good from distance and provides great rendering for scientific symbols. 
            Unfortunately <a href="https://github.com/blobject/agave">the last commit</a> was 4 years ago and I dont think it's gonna get any active maintenance any time soon.</p>
            <p>04. <a href="https://github.com/subframe7536/maple-font">Maple:</a> A monospaced coding font with some handwritten aesthetics. Not my personal favorite, but it def earns its place for its uniqueness.</p>
        </div>

        <h2>Music</h2>
        <p>Request this url with the <c>?r</c> tag to get a random link from below. Use it like this for even more fun: <br><c>curl "http://slb.sh/rocks?r" | yt-dlp -o - -a - | mpv -</c></p>
        <p>If you want an album added to this list feel free to send me recommendations!</p>
        <div class=indent>
            {list}
        </div>
    </div>
</body>
"#), StatusCode::OK)
}
