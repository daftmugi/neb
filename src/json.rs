use std::fs;
use std::path::Path;
use std::process;

use serde_json::Value;

pub fn read_file(json_path: &Path) -> Value {
  println!("==> Reading Knossos repo file...");

  let json_file = fs::read(&json_path).unwrap_or_else(|_| {
    println!("File read error: {}", &json_path.display());
    process::exit(1);
  });

  serde_json::from_slice(&json_file).unwrap_or_else(|_| {
    println!("Error parsing JSON");
    process::exit(1);
  })
}
