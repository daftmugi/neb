use std::path::Path;

use crate::{downloader, json, repo, web};

pub fn fetch(json_path: &Path) {
  downloader::fetch(json_path);
}

pub fn fetch_update(db_path: &Path, json_path: &Path) {
  fetch(json_path);
  update(db_path, json_path);
}

pub fn update(db_path: &Path, json_path: &Path) {
  let conn = repo::open_read_write(db_path);
  let json = json::read_file(json_path);
  let mods = json["mods"].as_array().unwrap();
  repo::update(&conn, mods);
}

pub fn list(db_path: &Path) {
  let conn = repo::open_read_only(db_path);
  repo::list(&conn);
}

pub fn list_json(db_path: &Path) {
  let conn = repo::open_read_only(db_path);
  repo::list_json(&conn);
}

pub fn search(db_path: &Path, query: &String) {
  let conn = repo::open_read_only(db_path);
  repo::search(&conn, query);
}

pub fn versions(db_path: &Path, mid: &String) {
  let conn = repo::open_read_only(db_path);
  repo::versions(&conn, mid);
}

pub fn json(db_path: &Path, mid: &String, version: &Option<String>) {
  let conn = repo::open_read_only(db_path);
  repo::json(&conn, mid, version);
}

pub fn cmdline(db_path: &Path, mid: &String, version: &Option<String>) {
  let conn = repo::open_read_only(db_path);
  repo::cmdline(&conn, mid, version);
}

pub fn modline(db_path: &Path, mid: &String, version: &Option<String>) {
  let conn = repo::open_read_only(db_path);
  repo::modline(&conn, mid, version);
}

pub fn sha256sum(db_path: &Path, mid: &String, version: &Option<String>) {
  let conn = repo::open_read_only(db_path);
  repo::sha256sum(&conn, mid, version);
}

pub fn dlsize(db_path: &Path, mid: &String, version: &Option<String>) {
  let conn = repo::open_read_only(db_path);
  repo::dlsize(&conn, mid, version);
}

pub fn web(db_path: &Path) {
  web::start(db_path);
}
