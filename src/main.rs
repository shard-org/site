use foxhole::{
    App, Http1, Router,
    Method::*, Response,
    resolve::{
        Url, UrlCollect, Query,
        ArgMap as Args,
    },
    action::Html,
    http::Response as HttpResponse,
    TypeCacheKey, TypeCache,
};

mod funcs;

struct CacheLen;
impl TypeCacheKey for CacheLen {
    type Value = u16;
}


use std::sync::{Arc, RwLock};
use std::collections::HashMap;
struct Files(HashMap<String, Arc<Vec<u8>>>);
impl TypeCacheKey for Files {
    type Value = Arc<RwLock<Files>>;
}

pub trait RwLockFileExt {
    fn get(&self, filename: &str) -> std::io::Result<Arc<Vec<u8>>>;
}

impl RwLockFileExt for RwLock<Files> {
    fn get(&self, filename: &str) -> std::io::Result<Arc<Vec<u8>>> {
        if let Some(contents) = self.read().unwrap().0.get(filename) {
            return Ok(contents.clone());
        }

        let contents = Arc::new(file(filename)?);
        self.write().unwrap().0
            .insert(String::from(filename), contents.clone());
        Ok(contents)
    }
}


#[allow(unused_parens)]
fn main() {
    let addr = std::env::args().skip(1).next()
        .unwrap_or(String::from("127.0.0.1:8080"));

    let scope = Router::new()
/* ##FUNC_START## */

/* ##FUNC_END## */
        .add_route("/*", Get(get_file))
        .fallback(|| file("files/not_found.html").map(|r| response(r, 404)).ok());

    let mut cache = TypeCache::new();
    cache.insert::<CacheLen>(std::env::var("CACHELEN").map_or(360, |s| s.parse::<u16>().unwrap()));
    cache.insert::<Files>(Arc::new(RwLock::new(Files(HashMap::new()))));

    App::builder(scope)
        .cache(cache)
        .run::<Http1>(addr);
}

fn get_file(Query(cache): Query<CacheLen>, Query(files): Query<Files>, Url(url): Url) -> Option<Response> {
    if url.starts_with('.') {
        return None;
    }

    let file = files.get(&format!("files{url}"));
    file.map_err(|e| {println!("{e}; for: {url:?}"); e})
        .map(|f| HttpResponse::builder()
            .header("cache-control", &format!("max-age={cache}, public"))
            .body((*f).clone())
            .unwrap()
        ).ok()
}

use std::io::Read;
pub fn file(filename: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = std::fs::File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    return Ok(buffer);
}

fn response<T: Into<Vec<u8>>>(chunk: T, code: u16) -> Response {
    let mut res = Response::new(chunk.into());
    *res.status_mut() = code.try_into().unwrap();
    res
}
