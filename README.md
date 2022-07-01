# README

## Installation

(Tested on macOS and Linux. May not work on Windows.)

1. Download and install Rust
    - <https://www.rust-lang.org/learn/get-started>
2. Download this git repo
3. Compile files
    ```sh
    cd neb
    cargo build --release
    ```

   NOTE: If you do not use `--release`, the resulting debug build is
   significantly slower.
4. Copy `./target/release/neb` wherever you want.


## Installation on Linux

You may need to install the following packages on Ubuntu:

```sh
apt install build-essential pkg-config libssl-dev
```


## Download Knossos Repo from FSNebula

```sh
neb fetch repo.json
```


## Initialize or Update Local Database

```sh
neb update repo.db repo.json
```

Neb is fast. On my machine, it initialized repo.db in 2.5
seconds. Updates took 1.5 seconds.


## Commands

See Usage below.

Neb is fast. On my machine, neb commands take 8 ms or less.


## (Optional) Web View

* Start web server
    ```sh
    neb web repo.db
    ```
* Open `http://localhost:3200` in your web browser.


## Usage

```
Usage: neb COMMAND [ARGS]

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

      BIND=0.0.0.0 PORT=3000 neb web repo.db


EXAMPLES

  # Update database
  neb update repo.db repo.json

  # Update temporary in-memory database (SQLite3 feature)
  neb update ':memory:' repo.json

  # Print list of mods as titles in plaint text
  neb list repo.db

  # Print list of mods as JSON
  neb list-json repo.db

  # Print mid and title of query
  neb search repo.db silent

  # Print list of versions of mod id
  neb versions repo.db

  # Print mod.json of mod by mid of latest version
  neb json repo.db str

  # Print mod.json of mod by mid of version 1.6.0
  neb json repo.db str 1.6.0

  # Print the command-line options of mod by mid
  neb cmdline repo.db MVPS

  # Print the command-line options of mod by mid and version
  neb cmdline repo.db MVPS 4.5.1

  # Print the mod params of mod by mid
  neb mod repo.db MVPS

  # Print the mod params of mod by mid and version
  neb mod repo.db MVPS 4.5.1

  # Print sha256sums of mod files by mid (mod id)
  neb sha256sum repo.db MVPS

  # Print sha256sums of mod files by mid (mod id) and version
  neb sha256sum repo.db MVPS 4.5.1

  # Print download size of mod files by mid (mod id)
  neb dlsize repo.db MVPS

  # Print download size of mod files by mid (mod id) and version
  neb dlsize repo.db MVPS 4.5.1

  # Download remote repo json file
  neb fetch repo.json

  # Download remote repo and update local db
  neb fetch-update repo.db repo.json
    # same as =>
    #   neb fetch repo.json
    #   neb update repo.db repo.json

  # Start web server
  neb web repo.db
```
