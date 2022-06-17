#![deny(warnings)]
use std::convert::Infallible;
use std::fmt::Write;
use std::path::Path;
use std::sync::Arc;

use http::Uri;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;
use tinytemplate::TinyTemplate;
use tokio::sync::Mutex;
use warp::{http::Response, Filter};

use crate::repo::{self, Mod};

static DEFAULT_BIND: [u8; 4] = [127, 0, 0, 1];
static DEFAULT_PORT: u16 = 3200;

type Db = Arc<Mutex<Connection>>;

#[derive(Serialize)]
struct ModListContext {
  mods: Vec<Mod>,
}

#[derive(Serialize)]
struct ModInfoContext<'a> {
  m: Value,
  mid: String,
  versions: &'a Vec<Version>,
  url_path: String,
  is_total_conversion: bool,
  packages: &'a Vec<Package>,
  total_size: u64,
  sha256sum: String,
  dependencies: Vec<(&'a String, &'a Vec<Value>)>,
  modline: String,
}

#[derive(Serialize)]
struct Version {
  text: String,
  is_selected: bool,
}

#[derive(Serialize)]
struct Package {
  name: String,
  status: String,
  notes: String,
  filename: String,
  filesize: u64,
  checksum: String,
  urls: Vec<Value>,
  dependencies: Vec<Value>,
}

#[tokio::main]
pub async fn start(db_path: &Path) {
  if std::env::var_os("RUST_LOG").is_none() {
    std::env::set_var("RUST_LOG", "mods=info");
  }

  env_logger::init();

  let conn = repo::open_read_only(db_path);
  let db = Arc::new(Mutex::new(conn));
  let bind = get_bind_env_var();
  let port = get_port_env_var();

  let favicon = warp::path!("favicon.ico").map(|| {
    Response::builder()
      .header("content-type", "image/png")
      .body(FAVICON.to_vec())
  });

  let index = warp::path::end().map(|| warp::redirect(Uri::from_static("/mods")));

  let about_page = warp::path!("about").map(|| warp::reply::html(ABOUT_PAGE));

  let mod_list_page = warp::path!("mods")
    .and(with_db(db.clone()))
    .and_then(list_page);

  let mod_info_page = warp::path!("mods" / String)
    .and(with_db(db.clone()))
    .and_then(info_page_without_version);

  let mod_info_page_as_json = warp::path!("mods" / String / "mod.json")
    .and(with_db(db.clone()))
    .and_then(info_page_without_version_as_json);

  let mod_info_page_with_version = warp::path!("mods" / String / String)
    .and(with_db(db.clone()))
    .and_then(info_page_with_version);

  let mod_info_page_with_version_as_json = warp::path!("mods" / String / String / "mod.json")
    .and(with_db(db.clone()))
    .and_then(info_page_with_version_as_json);

  let style_css = warp::path!("style.css").map(|| reply_css(STYLE_CSS));
  let about_css = warp::path!("index.css").map(|| reply_css(ABOUT_CSS));
  let mod_list_css = warp::path!("mod_list.css").map(|| reply_css(MOD_LIST_CSS));
  let mod_info_css = warp::path!("mod_info.css").map(|| reply_css(MOD_INFO_CSS));

  let mod_list_js = warp::path!("mod_list.js").map(|| reply_js(MOD_LIST_JS));
  let mod_info_js = warp::path!("mod_info.js").map(|| reply_js(MOD_INFO_JS));

  let bbparser_js = warp::path!("bbparser.js").map(|| reply_js(BBPARSER_JS));
  let xbbcode_js = warp::path!("xbbcode.js").map(|| reply_js(XBBCODE_JS));

  let routes = warp::get()
    .and(
      favicon
        .or(style_css)
        .or(about_css)
        .or(mod_list_css)
        .or(mod_info_css)
        .or(mod_list_js)
        .or(mod_info_js)
        .or(bbparser_js)
        .or(xbbcode_js)
        .or(index)
        .or(about_page)
        .or(mod_list_page)
        .or(mod_info_page)
        .or(mod_info_page_as_json)
        .or(mod_info_page_with_version)
        .or(mod_info_page_with_version_as_json)
        .or(warp::any().map(|| {
          warp::reply::with_status(
            warp::reply::html(NOT_FOUND_PAGE),
            http::StatusCode::NOT_FOUND,
          )
        })),
    )
    .with(warp::log("mods"));

  println!("Running server at http://localhost:{}", port);
  warp::serve(routes).run((bind, port)).await;
}

async fn list_page(db: Db) -> Result<impl warp::Reply, Infallible> {
  let conn = db.lock().await;
  let mods = repo::list_mods(&conn);

  let ctx = ModListContext { mods };

  let mut tt = TinyTemplate::new();
  tt.add_template("mod_list", MOD_LIST_PAGE).unwrap();
  let html = tt.render("mod_list", &ctx).unwrap();
  Ok(warp::reply::html(html))
}

async fn info_page_without_version(mid: String, db: Db) -> Result<impl warp::Reply, Infallible> {
  info_page(mid, None, db).await
}

async fn info_page_without_version_as_json(
  mid: String,
  db: Db,
) -> Result<impl warp::Reply, Infallible> {
  info_page_as_json(mid, None, db).await
}

async fn info_page_with_version(
  mid: String,
  version: String,
  db: Db,
) -> Result<impl warp::Reply, Infallible> {
  info_page(mid, Some(version), db).await
}

async fn info_page_with_version_as_json(
  mid: String,
  version: String,
  db: Db,
) -> Result<impl warp::Reply, Infallible> {
  info_page_as_json(mid, Some(version), db).await
}

async fn info_page_as_json(
  mid: String,
  version: Option<String>,
  db: Db,
) -> Result<impl warp::Reply, Infallible> {
  let conn = db.lock().await;

  if let Some(m) = repo::get_mod(&conn, &mid, &version) {
    let mod_json: Value = serde_json::from_str(m.mod_json.as_str()).unwrap();

    Ok(warp::reply::with_status(
      warp::reply::json(&mod_json),
      http::StatusCode::OK,
    ))
  } else {
    Ok(warp::reply::with_status(
      warp::reply::json(&serde_json::json!({})),
      http::StatusCode::NOT_FOUND,
    ))
  }
}

async fn info_page(
  mid: String,
  version: Option<String>,
  db: Db,
) -> Result<impl warp::Reply, Infallible> {
  let conn = db.lock().await;
  let m = if let Some(md) = repo::get_mod(&conn, &mid, &version) {
    md
  } else {
    return Ok(warp::reply::with_status(
      warp::reply::html(NOT_FOUND_PAGE.to_string()),
      http::StatusCode::NOT_FOUND,
    ));
  };

  let url_path = if let Some(v) = version {
    format!("/mods/{}/{}", mid, v)
  } else {
    format!("/mods/{}", mid)
  };

  let mod_json: Value = serde_json::from_str(m.mod_json.as_str()).unwrap();

  let is_total_conversion = mod_json["type"] == "tc";

  let mut packages: Vec<Package> = Vec::new();

  for p in mod_json["packages"].as_array().unwrap() {
    let file = &p["files"][0];

    packages.push(Package {
      name: p["name"].as_str().unwrap().to_string(),
      status: p["status"].as_str().unwrap().to_string(),
      notes: p["notes"].as_str().unwrap().to_string(),
      filename: file["filename"].as_str().unwrap().to_string(),
      filesize: file["filesize"].as_u64().unwrap(),
      checksum: file["checksum"][1].as_str().unwrap().to_string(),
      urls: file["urls"].as_array().unwrap().to_vec(),
      dependencies: p["dependencies"].as_array().unwrap().to_vec(),
    });
  }

  let total_size: u64 = packages.iter().map(|p| p.filesize).sum();
  let sha256sum: String = packages
    .iter()
    .map(|p| format!("{} {}", p.checksum, p.filename))
    .collect::<Vec<String>>()
    .join("\n");

  let dependencies: Vec<(&String, &Vec<Value>)> = packages
    .iter()
    .filter_map(|p| {
      if p.dependencies.is_empty() {
        None
      } else {
        Some((&p.name, &p.dependencies))
      }
    })
    .collect();

  let modline = format!(
    "-mod {}",
    mod_json["mod_flag"]
      .as_array()
      .unwrap()
      .iter()
      .map(|x| x.as_str().unwrap().to_string())
      .collect::<Vec<String>>()
      .join(",")
  );

  let versions = m
    .versions
    .iter()
    .map(|v| Version {
      text: v.clone(),
      is_selected: v == &m.version,
    })
    .collect();

  let ctx = ModInfoContext {
    m: mod_json,
    mid,
    versions: &versions,
    url_path,
    is_total_conversion,
    packages: &packages,
    total_size,
    sha256sum,
    dependencies,
    modline,
  };

  let mut tt = TinyTemplate::new();
  tt.add_template("mod_info", MOD_INFO_PAGE).unwrap();
  tt.add_formatter("bytes", format_bytes);
  tt.add_formatter("hostname", format_hostname);
  let html = tt.render("mod_info", &ctx).unwrap();
  Ok(warp::reply::with_status(
    warp::reply::html(html),
    http::StatusCode::OK,
  ))
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
  warp::any().map(move || db.clone())
}

fn reply_css(css: &str) -> Result<Response<&str>, http::Error> {
  Response::builder()
    .header("content-type", "text/css")
    .body(css)
}

fn reply_js(js: &str) -> Result<Response<&str>, http::Error> {
  Response::builder()
    .header("content-type", "application/javascript")
    .body(js)
}

fn get_bind_env_var() -> [u8; 4] {
  if let Ok(val) = std::env::var("BIND") {
    let list: Vec<u8> = val
      .split('.')
      .map(|x| {
        x.parse().unwrap_or_else(|_| {
          println!("Invalid BIND env variable.");
          std::process::exit(1)
        })
      })
      .collect();

    if list.len() != 4 {
      println!("Invalid BIND env variable.");
      std::process::exit(1)
    }

    [list[0], list[1], list[2], list[3]]
  } else {
    DEFAULT_BIND
  }
}

fn get_port_env_var() -> u16 {
  if let Ok(val) = std::env::var("PORT") {
    val.parse::<u16>().unwrap_or_else(|_| {
      println!("Invalid PORT env variable.");
      std::process::exit(1)
    })
  } else {
    DEFAULT_PORT
  }
}

fn format_bytes(value: &Value, output: &mut String) -> tinytemplate::error::Result<()> {
  let kb = 1024;
  let mb = kb * 1024;
  let gb = mb * 1024;

  match value {
    Value::Number(n) => {
      let n = n.as_u64().unwrap();

      let bytes = if n < kb {
        format!("{}", n)
      } else if n < mb {
        format!("{} KB", n / kb)
      } else if n < gb {
        format!("{} MB", n / mb)
      } else {
        format!("{} GB", n / gb)
      };

      write!(output, "{}", bytes)?;
      Ok(())
    }
    _ => Err(tinytemplate::error::Error::GenericError {
      msg: "Could not format bytes into string".to_string(),
    }),
  }
}

fn format_hostname(value: &Value, output: &mut String) -> tinytemplate::error::Result<()> {
  match value {
    Value::String(s) => {
      let uri = s.parse::<Uri>().unwrap();
      let hostname = uri.host().unwrap();

      write!(output, "{}", hostname)?;
      Ok(())
    }
    _ => Err(tinytemplate::error::Error::GenericError {
      msg: "Could not format URL".to_string(),
    }),
  }
}

// NOTE: Files were not moved to their own directories, because I'm
// not sure if Windows will accept forward slashes in paths.
static FAVICON: &[u8; 59531] = include_bytes!("favicon.png");
static STYLE_CSS: &str = include_str!("style.css");
static ABOUT_CSS: &str = include_str!("about.css");
static ABOUT_PAGE: &str = include_str!("about.html");
static MOD_LIST_CSS: &str = include_str!("list.css");
static MOD_LIST_JS: &str = include_str!("list.js");
static MOD_LIST_PAGE: &str = include_str!("list.html");
static MOD_INFO_CSS: &str = include_str!("info.css");
static MOD_INFO_JS: &str = include_str!("info.js");
static NOT_FOUND_PAGE: &str = include_str!("not_found.html");
static MOD_INFO_PAGE: &str = include_str!("info.html");
static BBPARSER_JS: &str = include_str!("bbparser.js");
static XBBCODE_JS: &str = include_str!("xbbcode.js");
