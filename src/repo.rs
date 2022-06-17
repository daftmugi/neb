use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process;

use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{named_params, CachedStatement, Connection, OpenFlags, Row};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Serialize)]
pub struct Mod {
  pub mid: String,
  pub version: String,
  pub versions: Vec<String>,
  pub title: String,
  pub tile: String,
  pub first_release: String,
  pub last_update: String,
  pub mod_json: String,
}

#[derive(Debug, PartialEq)]
struct FileMeta {
  name: String,
  size: u64,
  sha256: String,
}

fn ensure_sqlite3_db(path: &Path) {
  let mut f = File::open(path).unwrap();
  let mut buffer = [0; 15];

  f.read_exact(&mut buffer).unwrap_or_else(|_| {
    println!("Not a DB file: {}", path.display());
    process::exit(1);
  });

  if &buffer != b"SQLite format 3" {
    println!("Not a DB file: {}", path.display());
    process::exit(1);
  }
}

pub fn open_read_only(path: &Path) -> Connection {
  if path.exists() {
    ensure_sqlite3_db(path);
  } else {
    println!("File not found: {}", path.display());
    process::exit(1);
  }

  Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap_or_else(|_| {
    println!("DB connection open error: {}", path.display());
    process::exit(1);
  })
}

pub fn open_read_write(path: &Path) -> Connection {
  let path_exists = path.exists();
  let need_create = !path_exists;

  if path_exists {
    ensure_sqlite3_db(path);
  }

  let conn = Connection::open(&path).unwrap_or_else(|_| {
    println!("DB connection open error: {}", path.display());
    process::exit(1);
  });

  if need_create {
    create_db(&conn);
  }

  conn
}

fn create_db(conn: &Connection) {
  println!("==> Creating DB...");

  if conn.execute_batch(CREATE_TABLE_STMTS).is_err() {
    println!("DB create table error");
    process::exit(1);
  }
}

fn get_all_mod_ids_set(conn: &Connection) -> HashSet<[String; 2]> {
  let mut set = HashSet::new();

  let mut select = conn.prepare(LIST_MID_AND_VERSION_STMT).unwrap();
  let mut rows = select.query([]).unwrap();

  while let Some(row) = rows.next().unwrap() {
    let mid: String = row.get(0).unwrap();
    let version: String = row.get(1).unwrap();
    set.insert([mid, version]);
  }

  set
}

fn delete_mods(
  conn: &Connection,
  stored_ids: &HashSet<[String; 2]>,
  json_ids: &HashSet<[String; 2]>,
) {
  let mut select_stmt = prepare_select_statement(conn);
  let mut delete_stmt = conn.prepare_cached(DELETE_STMT).unwrap_or_else(|_| {
    println!("Prepare delete statement error");
    process::exit(1);
  });

  for to_delete in stored_ids.difference(json_ids) {
    let m = Mod {
      mid: to_delete[0].to_string(),
      version: to_delete[1].to_string(),
      versions: Vec::new(),
      title: "".to_string(),
      tile: "".to_string(),
      first_release: "".to_string(),
      last_update: "".to_string(),
      mod_json: "".to_string(),
    };

    let m_stored = select_mod(&mut select_stmt, &m).unwrap();

    println!("[DELETE] {} ({})", m_stored.title, m_stored.version);
    delete_stmt
      .execute(named_params! { ":mid": m.mid, ":version": m.version })
      .unwrap();
  }
}

pub fn update(conn: &Connection, mods_json: &Vec<Value>) {
  println!("==> Updating local mods database...");

  let mut select_stmt = prepare_select_statement(conn);
  let mut insert_stmt = prepare_insert_statement(conn);
  let mut update_stmt = prepare_update_statement(conn);

  let stored_ids = get_all_mod_ids_set(conn);
  let first_update = stored_ids == HashSet::new();
  let mut json_ids = HashSet::new();
  let progress_bar = create_progress_bar(mods_json);

  for mod_json in mods_json {
    let m = mod_from_serde_json(mod_json);
    json_ids.insert([m.mid.to_string(), m.version.to_string()]);

    match select_mod(&mut select_stmt, &m) {
      Some(m_stored) => {
        if m != m_stored {
          println!("[UPDATE] {} ({})", m.title, m.version);
          update_mod(&mut update_stmt, &m);
        }
      }

      None => {
        if !first_update {
          println!("[ADD]    {} ({})", m.title, m.version);
        }
        insert_mod(&mut insert_stmt, &m);
      }
    }

    if first_update {
      progress_bar.inc(1);
    }
  }

  if first_update {
    progress_bar.finish();
  }

  delete_mods(conn, &stored_ids, &json_ids);
}

pub fn list(conn: &Connection) {
  let mut select = conn.prepare(LIST_STMT).unwrap();
  let mut rows = select.query([]).unwrap();

  while let Some(row) = rows.next().unwrap() {
    let m = mod_from_sql_row(row);
    println!("{}", m.title);
  }
}

pub fn list_mods(conn: &Connection) -> Vec<Mod> {
  let mut select = conn.prepare(LIST_STMT).unwrap();
  let mut rows = select.query([]).unwrap();

  let mut list = Vec::new();
  while let Some(row) = rows.next().unwrap() {
    let m = mod_from_sql_row(row);
    list.push(m);
  }

  list
}

pub fn list_json(conn: &Connection) {
  let mut select = conn.prepare(LIST_STMT).unwrap();
  let mut rows = select.query([]).unwrap();

  let mut list = Vec::new();
  while let Some(row) = rows.next().unwrap() {
    let m = mod_from_sql_row(row);

    list.push(json!({
      "mid": m.mid,
      "title": m.title,
      "poster_url": m.tile,
    }));
  }

  println!("{}", json!({ "mods": list }));
}

pub fn search(conn: &Connection, query: &String) {
  let mut select = conn.prepare(SEARCH_STMT).unwrap();
  let mut rows = select
    .query(named_params! {":query": format!("%{}%", query)})
    .unwrap();

  let mut min_width = 15;
  let mut results = Vec::new();
  while let Some(row) = rows.next().unwrap() {
    let m = mod_from_sql_row(row);

    if m.mid.len() > min_width {
      min_width = m.mid.len();
    }

    results.push(m);
  }

  for m in results {
    println!("{:<width$}  {}", m.mid, m.title, width = min_width);
  }
}

pub fn versions(conn: &Connection, mid: &String) {
  let sorted_versions = get_sorted_versions(conn, mid);

  for v in sorted_versions {
    println!("{}", v);
  }
}

fn version_cmp(a: &String, b: &String) -> Ordering {
  if a == b {
    return Ordering::Equal;
  }

  let mut a_build: Vec<&str> = a.split('-').collect();
  let mut b_build: Vec<&str> = b.split('-').collect();

  if a_build.len() < 2 {
    a_build.push("~");
  }
  if b_build.len() < 2 {
    b_build.push("~");
  }

  if a_build[0] == b_build[0] {
    if a_build[1] < b_build[1] {
      return Ordering::Less;
    } else {
      return Ordering::Greater;
    }
  }

  let a_version = a_build[0].split('.').map(|x| x.parse::<u32>().unwrap_or(0));
  let b_version = b_build[0].split('.').map(|x| x.parse::<u32>().unwrap_or(0));

  a_version.cmp(b_version)
}

fn get_sorted_versions(conn: &Connection, mid: &String) -> Vec<String> {
  let mut select = conn.prepare(SELECT_VERSIONS_STMT).unwrap();
  let mut rows = select.query(named_params! {":mid": mid}).unwrap();
  let mut versions: Vec<String> = Vec::new();

  while let Some(row) = rows.next().unwrap() {
    versions.push(row.get(0).unwrap());
  }

  versions.sort_by(version_cmp);
  versions.reverse();
  versions
}

pub fn get_mod(conn: &Connection, mid: &String, version: &Option<String>) -> Option<Mod> {
  let sorted_versions = get_sorted_versions(conn, mid);

  if sorted_versions.is_empty() {
    return None;
  }

  let version: String = if let Some(ver) = version {
    ver.to_string()
  } else {
    sorted_versions[0].clone()
  };

  let mut select = conn.prepare(SELECT_STMT).unwrap();
  let mut rows = select
    .query(named_params! {":mid": mid, ":version": version})
    .unwrap();

  rows.next().unwrap().map(|r| {
    let mut m = mod_from_sql_row(r);
    m.versions = sorted_versions;
    m
  })
}

pub fn json(conn: &Connection, mid: &String, version: &Option<String>) {
  if let Some(m) = get_mod(conn, mid, version) {
    let mod_json: Value = serde_json::from_str(m.mod_json.as_str()).unwrap();
    println!("{}", serde_json::to_string_pretty(&mod_json).unwrap());
  } else {
    println!("Not found");
  }
}

pub fn cmdline(conn: &Connection, mid: &String, version: &Option<String>) {
  if let Some(m) = get_mod(conn, mid, version) {
    let mod_json: Value = serde_json::from_str(m.mod_json.as_str()).unwrap();
    println!("{}", mod_json["cmdline"].as_str().unwrap());
  }
}

pub fn modline(conn: &Connection, mid: &String, version: &Option<String>) {
  if let Some(m) = get_mod(conn, mid, version) {
    let mod_json: Value = serde_json::from_str(m.mod_json.as_str()).unwrap();
    let mod_flag_array = mod_json["mod_flag"].as_array().unwrap();
    let mod_flag: Vec<String> = mod_flag_array
      .iter()
      .map(|x| x.as_str().unwrap().to_string())
      .collect();
    let mod_string = mod_flag.join(",");

    println!("-mod {}", mod_string);
  }
}

pub fn get_sha256sum(conn: &Connection, mid: &String, version: &Option<String>) -> String {
  let files = filemeta(conn, mid, version);
  let mut lines: Vec<String> = Vec::new();

  for file in files {
    lines.push(format!("{} {}", file.sha256, file.name));
  }

  lines.join("\n")
}

pub fn sha256sum(conn: &Connection, mid: &String, version: &Option<String>) {
  println!("{}", get_sha256sum(conn, mid, version));
}

pub fn dlsize(conn: &Connection, mid: &String, version: &Option<String>) {
  let files = filemeta(conn, mid, version);
  let mut total = 0;

  for file in files {
    total += file.size;
    println!("{:<50} {:>20}", file.name, file.size);
  }
  println!("\n{:<50} {:>20}", "TOTAL", total);
}

fn filemeta(conn: &Connection, mid: &String, version: &Option<String>) -> Vec<FileMeta> {
  if let Some(m) = get_mod(conn, mid, version) {
    let mod_json: Value = serde_json::from_str(m.mod_json.as_str()).unwrap();
    let packages_array = mod_json["packages"].as_array().unwrap();
    packages_array
      .iter()
      .flat_map(|p| {
        let file_info: Vec<FileMeta> = p["files"]
          .as_array()
          .unwrap()
          .iter()
          .map(|f| FileMeta {
            name: f["filename"].as_str().unwrap().to_string(),
            size: f["filesize"].as_u64().unwrap(),
            sha256: f["checksum"][1].as_str().unwrap().to_string(),
          })
          .collect();
        file_info
      })
      .collect()
  } else {
    Vec::new()
  }
}

fn prepare_select_statement(conn: &Connection) -> CachedStatement {
  conn.prepare_cached(SELECT_STMT).unwrap_or_else(|_| {
    println!("Prepare select statement error");
    process::exit(1);
  })
}

fn prepare_insert_statement(conn: &Connection) -> CachedStatement {
  conn.prepare_cached(INSERT_STMT).unwrap_or_else(|_| {
    println!("Prepare insert statement error");
    process::exit(1);
  })
}

fn prepare_update_statement(conn: &Connection) -> CachedStatement {
  conn.prepare_cached(UPDATE_STMT).unwrap_or_else(|_| {
    println!("Prepare update statement error");
    process::exit(1);
  })
}

fn mod_from_serde_json(md: &Value) -> Mod {
  Mod {
    mid: md["id"].as_str().unwrap().to_string(),
    version: md["version"].as_str().unwrap().to_string(),
    versions: Vec::new(),
    title: md["title"].as_str().unwrap().to_string(),
    tile: md["tile"].as_str().unwrap_or("").to_string(),
    first_release: md["first_release"].as_str().unwrap().to_string(),
    last_update: md["last_update"].as_str().unwrap().to_string(),
    mod_json: md.to_string(),
  }
}

fn mod_from_sql_row(row: &Row) -> Mod {
  Mod {
    mid: row.get(0).unwrap(),
    version: row.get(1).unwrap(),
    versions: Vec::new(),
    title: row.get(2).unwrap(),
    tile: row.get(3).unwrap_or_else(|_| "".to_string()),
    first_release: row.get(4).unwrap_or_else(|_| "".to_string()),
    last_update: row.get(5).unwrap_or_else(|_| "".to_string()),
    mod_json: row.get(6).unwrap_or_else(|_| "".to_string()),
  }
}

fn select_mod(select_stmt: &mut CachedStatement, m: &Mod) -> Option<Mod> {
  let mut rows = select_stmt
    .query(named_params! {":mid": m.mid, ":version": m.version})
    .unwrap();
  let maybe_row = rows.next().unwrap();
  maybe_row.map(mod_from_sql_row)
}

fn insert_mod(insert_stmt: &mut CachedStatement, m: &Mod) {
  let params = named_params! {
    ":mid":           m.mid,
    ":version":       m.version,
    ":title":         m.title,
    ":tile":          m.tile,
    ":first_release": m.first_release,
    ":last_update":   m.last_update,
    ":mod_json":      m.mod_json,
  };

  insert_stmt.execute(params).unwrap_or_else(|_| {
    println!(
      "DB insert mod error: (mid: {}, version: {})",
      m.mid, m.version
    );
    process::exit(1);
  });
}

fn update_mod(update_stmt: &mut CachedStatement, m: &Mod) {
  let params = named_params! {
    ":mid":           m.mid,
    ":version":       m.version,
    ":title":         m.title,
    ":tile":          m.tile,
    ":first_release": m.first_release,
    ":last_update":   m.last_update,
    ":mod_json":      m.mod_json,
  };

  update_stmt.execute(params).unwrap_or_else(|_| {
    println!(
      "DB update mod error: (mid: {}, version: {})",
      m.mid, m.version
    );
    process::exit(1);
  });
}

fn create_progress_bar(mods_json: &Vec<Value>) -> ProgressBar {
  let progress_bar = ProgressBar::new(mods_json.len().try_into().unwrap());
  progress_bar.set_style(
    ProgressStyle::default_bar()
      .template("[{wide_bar}] {pos}/{len}")
      .progress_chars("=> "),
  );
  progress_bar
}

static CREATE_TABLE_STMTS: &str = r#"
CREATE TABLE IF NOT EXISTS mods (
  id               INTEGER PRIMARY KEY,
  mid              TEXT NOT NULL,
  title            TEXT NOT NULL,
  tile             TEXT,
  version          TEXT NOT NULL,
  first_release    DATE,
  last_update      DATE,
  mod_json         JSON NOT NULL
);
CREATE UNIQUE INDEX mods_mid_version_unique_index ON mods (mid, version);
CREATE INDEX mods_title_index ON mods (title);
CREATE INDEX mods_first_release ON mods (first_release);
CREATE INDEX mods_last_update ON mods (last_update);
"#;

static INSERT_STMT: &str = r#"
INSERT
INTO mods (mid, version, title, tile, first_release, last_update, mod_json)
VALUES (:mid, :version, :title, :tile, :first_release, :last_update, :mod_json);
"#;

static DELETE_STMT: &str = r#"
DELETE FROM mods WHERE (mid = :mid) AND (version = :version);
"#;

static UPDATE_STMT: &str = r#"
UPDATE mods
SET title = :title,
    tile = :tile,
    first_release = :first_release,
    last_update = :last_update,
    mod_json = :mod_json
WHERE (mid = :mid) AND (version = :version);
"#;

static LIST_STMT: &str = r#"
SELECT mid, version, title, tile, max(last_update)
FROM mods
GROUP BY mid
ORDER BY title;
"#;

static LIST_MID_AND_VERSION_STMT: &str = r#"
SELECT mid, version FROM mods
"#;

static SELECT_STMT: &str = r#"
SELECT mid, version, title, tile, first_release, last_update, mod_json
FROM mods
WHERE (mid = :mid) AND (version = :version)
LIMIT 1;
"#;

static SELECT_VERSIONS_STMT: &str = r#"
SELECT version
FROM mods
WHERE (mid = :mid)
"#;

static SEARCH_STMT: &str = r#"
SELECT mid, version, title, tile, max(last_update)
FROM mods
WHERE (title LIKE :query)
GROUP BY mid
ORDER BY title;
"#;
