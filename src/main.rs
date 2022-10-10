use std::env;
use std::path::Path;
use std::process;

use neb::command;

fn main() {
  let argv_0 = env::args().nth(1).unwrap_or_else(|| "".to_string());
  let argv_1 = env::args().nth(2).unwrap_or_else(|| "".to_string());
  let argv_2 = env::args().nth(3).unwrap_or_else(|| "".to_string());
  let argv_3 = env::args().nth(4);

  match argv_0.as_str() {
    "--help" | "help" => print_help(),
    "--version" => print_version(),

    "fetch" => command::fetch(as_path(&argv_1)),

    "search" => command::search(as_path(&argv_1), as_string(&argv_2)),
    "versions" => command::versions(as_path(&argv_1), as_string(&argv_2)),

    "list" => command::list(as_path(&argv_1)),
    "list-json" => command::list_json(as_path(&argv_1)),

    "fetch-update" => command::fetch_update(as_path(&argv_1), as_path(&argv_2)),
    "update" => command::update(as_path(&argv_1), as_path(&argv_2)),

    "json" => command::json(as_path(&argv_1), as_string(&argv_2), &argv_3),
    "cmdline" => command::cmdline(as_path(&argv_1), as_string(&argv_2), &argv_3),
    "mod" => command::modline(as_path(&argv_1), as_string(&argv_2), &argv_3),
    "sha256sum" => command::sha256sum(as_path(&argv_1), as_string(&argv_2), &argv_3),
    "dlsize" => command::dlsize(as_path(&argv_1), as_string(&argv_2), &argv_3),

    "web" => command::web(as_path(&argv_1)),

    _ => {
      print_help();
      process::exit(1);
    }
  };
}

fn print_version() {
  let version = env!("CARGO_PKG_VERSION");
  println!("{}", version);
}

fn print_help() {
  let help = format!(
    r###"
Usage: {cmd_name} COMMAND [ARGS]

FLAGS

  --help                          : Print this message
  --version                       : Print version


COMMANDS

  help                            : Print this message
  fetch        JSON               : Download remote repo json file
  fetch-update REPO JSON          : Same as 'fetch' followed by 'update'
  update       REPO JSON          : Update repo database
  list         REPO               : Print list of mods as titles in plain text
  list-json    REPO               : Print list of mods as JSON
  search       REPO QUERY         : Print mod ids of queried title
  versions     REPO MID           : Print list of versions of mod id
  json         REPO MID [VERSION] : Print mod.json of mod id (default: latest)
  cmdline      REPO MID [VERSION] : Print command-line opts of mod id [and version]
  mod          REPO MID [VERSION] : Print mod params of mod id [and version]
  sha256sum    REPO MID [VERSION] : Print sha256sums of files by mod id [and version]
  dlsize       REPO MID [VERSION] : Print total download size by mod id [and version]
  web          REPO               : Start a web server to view mod info in the browser


WEB

  The "web" command supports BIND and PORT environment variables.

  BIND and PORT are optional. The default values are:

  BIND=127.0.0.1
  PORT=3200

  Example:

      BIND=0.0.0.0 PORT=3000 {cmd_name} web repo.db


EXAMPLES

  # Update database
  {cmd_name} update repo.db repo.json

  # Update temporary in-memory database (SQLite3 feature)
  {cmd_name} update ':memory:' repo.json

  # Print list of mods as titles in plaint text
  {cmd_name} list repo.db

  # Print list of mods as JSON
  {cmd_name} list-json repo.db

  # Print mid and title of query
  {cmd_name} search repo.db silent

  # Print list of versions of mod id
  {cmd_name} versions repo.db str

  # Print mod.json of mod by mid of latest version
  {cmd_name} json repo.db str

  # Print mod.json of mod by mid of version 1.6.0
  {cmd_name} json repo.db str 1.6.0

  # Print the command-line options of mod by mid
  {cmd_name} cmdline repo.db MVPS

  # Print the command-line options of mod by mid and version
  {cmd_name} cmdline repo.db MVPS 4.5.1

  # Print the mod params of mod by mid
  {cmd_name} mod repo.db MVPS

  # Print the mod params of mod by mid and version
  {cmd_name} mod repo.db MVPS 4.5.1

  # Print sha256sums of mod files by mid (mod id)
  {cmd_name} sha256sum repo.db MVPS

  # Print sha256sums of mod files by mid (mod id) and version
  {cmd_name} sha256sum repo.db MVPS 4.5.1

  # Print download size of mod files by mid (mod id)
  {cmd_name} dlsize repo.db MVPS

  # Print download size of mod files by mid (mod id) and version
  {cmd_name} dlsize repo.db MVPS 4.5.1

  # Download remote repo json file
  {cmd_name} fetch repo.json

  # Download remote repo and update local db
  {cmd_name} fetch-update repo.db repo.json
    # same as =>
    #   {cmd_name} fetch repo.json
    #   {cmd_name} update repo.db repo.json

  # Start web server
  {cmd_name} web repo.db
    "###,
    cmd_name = env!("CARGO_PKG_NAME")
  );

  println!("{}", help.trim());
}

fn as_path(arg: &String) -> &Path {
  if arg.is_empty() {
    print_help();
    process::exit(1);
  }

  Path::new(arg)
}

fn as_string(arg: &String) -> &String {
  if arg.is_empty() {
    print_help();
    process::exit(1);
  }

  arg
}
