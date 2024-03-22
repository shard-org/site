
#![allow(unused_imports)]
use hyper::{
    HeaderMap,
    header::HeaderValue,
    body::Bytes,
    Request, Response, StatusCode,
};
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
    if !NEWS_TOKENS.contains(&token) {
        return crate::Res(String::from("Invalid Token.. nice try :p"), StatusCode::UNAUTHORIZED);
    }

    if body.is_empty() {
        return crate::Res(String::from("Empty Body"), StatusCode::BAD_REQUEST);
    }

    let body = String::from_utf8_lossy(&body);
    NEWS.add(&body).await.unwrap();

    return crate::Res(String::new(), StatusCode::OK);
}


let news = NEWS.lock().await;
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
        None => news.last().unwrap(),
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

let mut sorted = news;
sorted.sort_by(|a, b| b.cmp(a)); // reverse sort (oldest first)
let news_str = sorted.iter().fold(String::new(), |mut acc, (t, m)| {
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




const NEWS_FILENAME: &str = "files/.news";
const NEWS_TOKENS: &[&str] = &[
    "MBrgJ15MF5jinC7KbgmZv7QhvtzRL4/Znr7R38PSH5k=", // anthony
];

use tokio::{
    sync::Mutex,
    io::AsyncWriteExt,
    fs::File,
};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local, TimeZone};

type News = Vec<(u64, Box<str>)>;

lazy_static::lazy_static! {
    static ref NEWS: Mutex<News> = {
        let mut file = std::fs::File::open(NEWS_FILENAME).unwrap();

        let mut content = String::new();
        let _ = file.read_to_string(&mut content).unwrap();

        NEWS::from(&content)
    };
}

impl NEWS {
    fn from(input: &str) -> Mutex<News> {
        let news = input.lines().fold(Vec::new(), |mut acc, l| {
            let (date, msg) = l.split_once(',')
                .expect("malformed config file");

            let date: u64 = date.parse()
                .expect("invalid date");

            acc.push((date, Box::from(msg))); acc
        });

        Mutex::new(news)
    }

    async fn add(&self, text: &str) -> tokio::io::Result<()> {
        // let time = chrono::Local::now()
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap().as_secs();

        let mut news = self.lock().await;
        news.push((time, Box::from(text)));
        drop(news);
        
        self.save().await
    }

    async fn save(&self) -> tokio::io::Result<()> {
        let news = self.lock().await;
        let contents = news.iter().fold(String::new(), |mut acc, (t, m)| {
            acc.push_str(&format!("{},{}\n", t, m)); acc
        });
        drop(news);
        
        let mut file = File::create(NEWS_FILENAME).await?;
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
let news = NEWS.lock().await;
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
            <p>- <r>email:</r> <i>ant@slb.sh</i></p>
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
format!(r#"
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
        <h2>Music</h2>
        <p>(Most are bandcamp links)</p>
        <div class=indent>
            <p>01. <a href="https://autonoesis.bandcamp.com/album/moon-of-foul-magics">Autonoesis - Moon of Foul Magics</a></p>
            <p>02. <a href="https://eosphoroscult.bandcamp.com/album/ii">Eosphoros - II</a></p>
            <p>03. <a href="https://mephorash.bandcamp.com/album/shem-ha-mephorash">Mephorash - Shem Ha Mephorash</a></p>
            <p>04. <a href="https://catacombesbl.bandcamp.com/album/des-glaires-et-des-briques">Catacombes - Des Des Glaires et des Briques</a></p>
            <p>05. <a href="https://gevurah.bandcamp.com/album/gehinnom">GEVURAH - Gehinnom</a></p>
            <p>06. <a href="https://tristetage.bandcamp.com/album/herbst-2022-ep">Triste Tage - Herbst 2022 EP</a></p>
            <p>07. <a href="https://malignantvoices.bandcamp.com/album/gloom-ash-and-emptiness-to-the-horizon">Ashes - Gloom Ash and Emptiness to the Horizon</a></p>
            <p>08. <a href="https://sevengill.bandcamp.com/album/sea">Sevengill - Sea</a></p>
            <p>09. <a href="https://seawitchdoom.bandcamp.com/album/the-blackened-sea">Sea Witch - The Blackened Sea</a></p>
            <p>0a. <a href="https://wyndbrln.bandcamp.com/album/wynd">WYND - WYND</a></p>
            <p>0b. <a href="https://omenstones.bandcamp.com/album/omen-stones-2">Omen Stones - Omen Stones</a></p>
            <p>0c. <a href="https://oprichnik.bandcamp.com/album/the-abyss-of-solitude">Oprichnik - The Abyss of Solitude</a></p>
            <p>0d. <a href="https://watainsom.bandcamp.com/album/sworn-to-the-dark">Watain - Sworn to the Dark</a></p>
            <p>0e. <a href="https://thurnin.bandcamp.com/album/menhir">Thurnin - Menhir</a></p>
            <p>0f. <a href="https://aawks.bandcamp.com/album/heavy-on-the-cosmic">AWWKS - Heavy on the Cosmic</a></p>
            <p>10. <a href="https://hazeshuttle.bandcamp.com/album/hazeshuttle">Hazeshuttle - Hazeshuttle</a></p>
            <p>11. <a href="https://dozerofficial.bandcamp.com/album/drifting-in-the-endless-void">Dozer - Drifting in the Endless Void</a></p>
            <p>12. <a href="https://ripplemusic.bandcamp.com/album/bury-the-hatchet-2">Shotgun Sawyer - Bury the Hatchet</a></p>
            <p>13. <a href="https://demiser.bandcamp.com/album/through-the-gate-eternal">Demiser - Through the Gate Eternal</a></p>
            <p>14. <a href="https://relapsealumni.bandcamp.com/album/death-is-this-communion">High on Fire - Death is This Communion</a></p>
            <p>15. <a href="https://patriciadallio.bandcamp.com/album/lencre-des-voix-secretes">Patricia Dallio - L'ENCRE DES VOIX SECRETES</a></p>
            <p>16. <a href="https://thierryzaboitzeff.bandcamp.com/album/prom-th-e-artists-edition">Thierry Zaboitzeff - Prométhée - Artist's edition</a></p>
            <p>17. <a href="https://blackwoodsband.bandcamp.com/album/landscapes">Black Woods - Landscapes</a></p>
            <p>18. <a href="https://xasthurband.bandcamp.com/album/defective-epitaph">Zasthur - Defective Epitaph</a></p>
            <p>19. <a href="https://satanath.bandcamp.com/album/sat365-septory-rotting-humanity-compilation-2023">Septory - Rotting Humanity</a></p>
            <p>1a. <a href="https://rustblackthrash.bandcamp.com/album/raw-shredding-death">Rust - Raw Shredding Death</a></p>
            <p>1b. <a href="https://lowriderofficial.bandcamp.com/album/refractions">Lowrider - Refractions</a></p>
            <p>1c. <a href="https://khonsu.bandcamp.com/album/the-xun-protectorate">Khonsu - The Xun Protectorate</a></p>
            <p>1d. <a href="https://primordialofficial.bandcamp.com/album/spirit-the-earth-aflame">Primordial - Spirit the Earth Aflame</a></p>
            <p>1e. <a href="https://ebonypendant.bandcamp.com/album/incantation-of-eschatological-mysticism">Ebony Pendant - Incantation Of Eschatological Mysticism</a></p>
            <p>1f. <a href="https://ebonypendant.bandcamp.com/album/garden-of-strangling-roots">Ebony Pendant - Garden Of Strangling Roots</a></p>
            <p>20. <a href="https://madeofstonerecordings.bandcamp.com/album/tortuga-deities">Tortuga - Deities</a></p>
            <p>21. <a href="https://thegraviators.bandcamp.com/album/motherload">The Graviators - Motherload</a></p>
            <p>22. <a href="https://gololedx.bandcamp.com/album/go-oled">Gołoledź - Gołoledź</a></p>
            <p>23. <a href="https://copperage.bandcamp.com/album/buerismo">Copper Age - Buerismo</a></p>
        </div>

        <h2>Software</h2>
        <div class=indent>
            <p>01. <a href="http://eradman.com/entrproject/">entr:</a> Runs any command whenever a chosen file changes. Useful for recompiling and running a project whenever you update the source.</p>
            <p>02. <a href="https://github.com/rvaiya/warpd">warpd:</a> Keyboard driven virtual pointer. Lets you fully control your mouse with the keyboard!</p>
            <p>03. <a href="https://github.com/tulir/gomuks">gomucks:</a> Terminal based matrix client. Includes image previews and emoji support.</p>
            <p>04. <a href="https://spade-lang.org/">spade:</a> A rusty HDL (hardware description language). Clean and actively maintained.</p>
            <p>05. <a href="https://typst.app/">typst:</a> Markup-based typesetting system, like LaTeX but much cleaner, elegant, and easier to learn.</p>
            <!-- <p>06. <a href=""></a> <p> -->
        </div>
    </div>
</body>
"#)
}
