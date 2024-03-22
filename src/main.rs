use hyper::{
    body::{Incoming, Bytes},

    service::service_fn,
    server::conn::http1,
    Request,
    Response,
    StatusCode,
};

use hyper_util::rt::TokioIo;
use http_body_util::{Full, combinators::BoxBody, BodyExt};

use std::{
    collections::HashMap,
    sync::Arc,
    convert::Infallible,
    io::Read,
    fs::File as StdFile,
};

use tokio::{
    net::TcpListener,
    sync::RwLock,
    fs::File,
    io::AsyncReadExt,
};

use once_cell::sync::Lazy;

pub type Error = Box<dyn std::error::Error + Sync + Send>;



mod args;
mod utils;
mod funcs;

use args::Conf;
use utils::HandleErr;


static CACHE_DISABLE: Lazy<bool> = Lazy::new(|| {
    if let Ok(value) = std::env::var("RP_CACHE") {
        if value.as_str() == "false" {
            return true;
        }
    }

    false
});


#[tokio::main]
async fn main() {
    let conf = Conf::parse(std::env::args().skip(1).collect()).unwrap_or_else(|e| {
        println!("\x1b[31;1mARG ERR:\x1b[0m {}", e);
        std::process::exit(1);
    });

    // call init
    // funcs::_init();

    let listener = TcpListener::bind((conf.addr, conf.port)).await.handle();

    loop {
        let (stream, _) = listener.accept().await.handle();
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request)).await
            {
                println!("\x1b[31;1mCONN ERR:\x1b[0m {}", e);
            }
        });
    }
}

async fn handle_request(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    let (parts, body) = req.into_parts();

    let body = body.collect().await.unwrap().to_bytes();

    let uri = parts.uri.to_string();
    let (uri, args) = parse_uri_args(&uri);

    let res = match parts.uri.path() {
/* ##FUNC_MATCH_START## */
"/ftoc" => funcs::ftoc(uri, args, parts.headers, body).await.into_response(),
"/news" => funcs::news(uri, args, parts.headers, body).await.into_response(),
"/time" => funcs::time(uri, args, parts.headers, body).await.into_response(),
"/" => funcs::index(uri, args, parts.headers, body).await.into_response(),
"/blog" => funcs::blog(uri, args, parts.headers, body).await.into_response(),
"/dice" => funcs::dice(uri, args, parts.headers, body).await.into_response(),
"/rocks" => funcs::rocks(uri, args, parts.headers, body).await.into_response(),

/* ##FUNC_MATCH_END## */
        name => {
            if name.starts_with("/.") {
                return Ok(not_found().await);
            }

            let name = String::from("files") + name;

            match file(&name).await {
                Ok(f) => f.into_response(),
                Err(_) => return Ok(not_found().await),
            }

        },
    };

    Ok(boxed(res))
}


fn parse_uri_args<'a>(uri: &'a str) -> (&'a str, HashMap<&'a str, String>) {
    let Some((uri, args)) = uri.split_once('?') else {
        return (uri, HashMap::new());
    };

    let args = args.split('&');

    let mut out_args = HashMap::new();
    for arg in args {
        let (key, value) = arg.split_once('=')
            .unwrap_or((arg, ""));

        let value = value.replace('+', " ");

        let value = urlencoding::decode(&value)
            .map(|v| v.into_owned())
            .unwrap_or(String::new());

        out_args.insert(key, value);
    }

    (uri, out_args)
}


pub async fn not_found() -> Response<BoxBody<Bytes, Infallible>> {
    const BACKUP: &[u8] = b"404 not_found page... not found... uh oh";

    let filename = std::env::var("RST_404").unwrap_or(String::from("files/not_found.html"));
    let res = file(&filename).await.unwrap_or(Arc::from(BACKUP));
    boxed(response(res, StatusCode::NOT_FOUND))
}



pub trait IntoResponse {
    fn into_response(self) -> Response<Bytes>;
}

impl<T: AsRef<[u8]>> IntoResponse for T {
    fn into_response(self) -> Response<Bytes> {
        response(self, StatusCode::OK)
    }
}

pub struct Res<T>(T, StatusCode);

impl<T: AsRef<[u8]>> IntoResponse for Res<T> {
    fn into_response(self) -> Response<Bytes> {
        response(self.0, self.1)
    }
}

fn boxed(res: Response<Bytes>) -> Response<BoxBody<Bytes, Infallible>> {
    let (parts, body) = res.into_parts();
    let body = Full::new(body).boxed();
    Response::from_parts(parts, body)
}


pub async fn file(filename: &str) -> Result<Arc<[u8]>, std::io::Error> {
    static FILE_CACHE: Lazy<RwLock<HashMap<String, Arc<[u8]>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

    if *CACHE_DISABLE {
        let mut file = File::open(filename).await?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        return Ok(Arc::from(buffer));
    }

    let cache = FILE_CACHE.read().await;
    if let Some(contents) = cache.get(filename) {
        return Ok(contents.clone());
    }
    drop(cache);


    let mut file = File::open(filename).await?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let contents: Arc<[u8]> = Arc::from(buffer);


    let mut cache = FILE_CACHE.write().await;
    cache.insert(filename.to_string(), contents.clone());
    drop(cache);

    Ok(contents)
}

pub fn file_nc(filename: &str) -> Result<Box<[u8]>, std::io::Error> {
    let mut file = StdFile::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer.into_boxed_slice())
}

fn response<T: AsRef<[u8]>>(chunk: T, code: StatusCode) -> Response<Bytes> {
    let bytes = chunk.as_ref().to_vec();
    Response::builder()
        .status(code)
        .body(bytes.into())
        .unwrap()
}
