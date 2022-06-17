use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process;
use std::str;

use curl::easy::Easy;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;

pub fn fetch(json_path: &Path) {
  let url = "https://fsnebula.org/storage/repo.json";

  let path = json_path.to_string_lossy();
  let header_path_string = format!("{}.header", path);
  let header_path = Path::new(&header_path_string);
  let json_part_path_string = format!("{}.part", path);
  let json_part_path = Path::new(&json_part_path_string);

  // DOWNLOAD HEADER

  let prev_header = fs::read_to_string(header_path).unwrap_or_else(|_| "".to_string());
  let curr_header = download_header(url);

  let re_etag = Regex::new(r#"(?mi)^etag: "(.+)""#).unwrap();
  let prev_etag = re_etag.captures(&prev_header).map(|cap| cap[1].to_string());
  let curr_etag = re_etag.captures(&curr_header).map(|cap| cap[1].to_string());

  if prev_etag == curr_etag {
    println!("Already most recent version.");
    return;
  }

  // DOWNLOAD BODY

  // Create placeholder files until "part" finishes downloading.
  create_placeholder_file(header_path);
  create_placeholder_file(json_path);

  download_json_file(url, json_part_path);

  fs::write(header_path, curr_header).unwrap_or_else(|_| {
    println!("Failed to write header file: {}", header_path.display());
  });
  fs::rename(json_part_path, json_path).unwrap_or_else(|_| {
    println!("Failed to move repo file to '{}'", json_path.display());
    process::exit(1);
  });
}

fn download_header(url: &str) -> String {
  let mut headers = Vec::new();
  let mut header_handle = Easy::new();
  header_handle.url(url).unwrap();
  header_handle.nobody(true).unwrap();

  let mut transfer = header_handle.transfer();
  transfer
    .header_function(|data| {
      headers.push(str::from_utf8(data).unwrap().to_string());
      true
    })
    .unwrap();

  transfer.perform().unwrap_or_else(|_| {
    println!("Failed to download header.");
  });

  drop(transfer);

  headers.join("")
}

fn create_placeholder_file(path: &Path) {
  if !path.exists() {
    File::create(path).unwrap_or_else(|_| {
      println!("Could not create file: {}", path.display());
      process::exit(1);
    });
  }
}

fn download_json_file(url: &str, json_part_path: &Path) {
  let mut json_part_file = File::create(json_part_path).unwrap();
  let mut download_handle = Easy::new();
  download_handle.url(url).unwrap();
  add_progress_bar(&mut download_handle);

  download_handle
    .write_function(move |data| {
      json_part_file.write_all(data).unwrap();
      Ok(data.len())
    })
    .unwrap();

  download_handle.perform().unwrap_or_else(|_| {
    println!("Download failed.");
    process::exit(1);
  });
}

fn add_progress_bar(download_handle: &mut Easy) {
  let progress_bar = ProgressBar::new(0);
  progress_bar.set_style(
    ProgressStyle::default_bar()
      .template("[{elapsed_precise}] [{wide_bar}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
      .progress_chars("=> "),
  );

  download_handle.progress(true).unwrap();
  download_handle
    .progress_function(move |dltotal, dlnow, _ultotal, _ulnow| {
      let now = dlnow as u64;
      let total = dltotal as u64;

      if progress_bar.length() != total && total > 0 {
        progress_bar.set_length(total);
      }

      if now > 0 {
        progress_bar.set_position(now);
      }
      true
    })
    .unwrap();
}
