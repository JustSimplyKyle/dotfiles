extern crate clap;

use clap::App;
use clap::Arg;
use home::home_dir;
use serde::Deserialize;
use std::fmt::Display;
use std::fs::create_dir_all;
use std::fs::remove_file;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::{
    env::set_current_dir,
    fs::File,
    io::{self, BufReader, Read},
    path::PathBuf,
};
use termion::color;

#[derive(Deserialize)]
struct Config {
    symlinks: Vec<String>,
}

// TODO: show a warning ("unreachable") for any git-tracked file that is not
//       symlinked and does not have a symlinked parent.
impl Config {
    fn read_from_path(path: impl AsRef<Path>) -> io::Result<Config> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // This bit can be simplified one this PR is merged:
        // https://github.com/alexcrichton/toml-rs/pull/452
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let config = toml::de::from_slice(&buf)?;

        Ok(config)
    }
}

struct Context {
    home: PathBuf,
    repo: PathBuf,
    config: Config,
}

impl Context {
    fn new(home: PathBuf) -> io::Result<Context> {
        Ok(Context {
            home: home.clone(),
            repo: home.join(".dotfiles/home"),
            config: Config::read_from_path(home.join(".dotfiles/dotfiles.toml"))?,
        })
    }

    fn home_path(&self, path: &Path) -> PathBuf {
        self.home.join(path)
    }

    fn repo_path(&self, path: &Path) -> PathBuf {
        self.repo.join(path)
    }

    fn get_linked_paths(&self) -> io::Result<Vec<Link>> {
        let mut paths = Vec::new();

        for dir in self.config.symlinks.iter() {
            paths.push(Link::new(PathBuf::from(dir), self));
        }

        paths.sort_unstable_by_key(|link| link.path.clone());
        Ok(paths)
    }

    fn state_for_path(&self, path: &Path) -> io::Result<PathState> {
        let home_path = self.home_path(path);

        let state = if home_path.exists() {
            if home_path.canonicalize()? == self.repo.join(path).canonicalize()? {
                PathState::Fine
            } else if home_path.is_symlink() {
                PathState::BadLink
            } else {
                PathState::Conflict
            }
        } else if !home_path.is_symlink() {
            PathState::Missing
        } else {
            PathState::Broken
        };

        Ok(state)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum PathState {
    Fine,     // Link points to dotfiles repo.
    Missing,  // File is missing.
    Broken,   // Link exists and is broken.
    BadLink,  // Link exists and points to another file.
    Conflict, // Not a link.
    Error(String),
}

impl PathState {
    fn colour(&self) -> &'static str {
        match self {
            PathState::Missing => color::Cyan.fg_str(),
            PathState::Broken => color::Yellow.fg_str(),
            PathState::BadLink => color::Magenta.fg_str(),
            PathState::Conflict => color::Red.fg_str(),
            PathState::Fine => color::Green.fg_str(),
            PathState::Error(_) => color::Red.fg_str(),
        }
    }
}

impl Display for PathState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_fmt(format_args!(
            "{}{:?}{}",
            &self.colour(),
            self,
            color::Reset.fg_str(),
        ));
    }
}

impl From<io::Result<PathState>> for PathState {
    fn from(item: io::Result<PathState>) -> Self {
        match item {
            Ok(state) => state,
            Err(err) => PathState::Error(err.to_string()),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Link {
    // Sorting happens based on field sorting from top to bottom.
    state: PathState,
    new_state: Option<PathState>,
    path: PathBuf,
}

impl Display for Link {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.new_state {
            Some(new_state) => formatter.write_fmt(format_args!(
                "{} -> {} {}",
                self.state,
                new_state,
                self.path.to_string_lossy(),
            )),
            None => formatter.write_fmt(format_args!(
                "{} {}",
                self.state,
                self.path.to_string_lossy(),
            )),
        }
    }
}

impl Link {
    fn new(path: PathBuf, context: &Context) -> Link {
        let state = context.state_for_path(&path);

        Link {
            path,
            state: state.into(),
            new_state: None,
        }
    }

    fn create_link(&mut self, context: &Context) {
        let link_name = context.home_path(&self.path);
        let parent_dir = link_name
            .parent()
            .expect("symlink file has a parent directory");

        self.new_state = Some(match create_dir_all(parent_dir) {
            Err(err) => PathState::Error(err.to_string()), // Directory creation failed.
            Ok(_) => match symlink(context.repo_path(&self.path), link_name) {
                Ok(()) => PathState::Fine,
                Err(err) => PathState::Error(err.to_string()),
            },
        });
    }

    fn apply(&mut self, context: &Context) {
        match self.state {
            PathState::Missing => {
                self.create_link(context);
            }
            PathState::Broken | PathState::BadLink => {
                let home_path = context.home_path(&self.path);
                match remove_file(home_path) {
                    Ok(()) => self.create_link(context),
                    Err(err) => self.new_state = Some(PathState::Error(err.to_string())),
                };
            }
            PathState::Conflict => {
                // TODO: Is there anything we CAN do here?
                // no-op
            }
            PathState::Fine | PathState::Error(_) => {
                // no-op
            }
        }
    }
}

fn main() {
    let matches = App::new("sysconfig")
        .version("1.0")
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .short("d")
                .help("Show plan but don't make any changes"),
        )
        .get_matches();

    let home = home_dir().expect("could find home dir");
    let context = Context::new(home).expect("context initialised");

    set_current_dir(&context.repo).expect("change directory into $REPO/home.");
    let mut paths = context.get_linked_paths().expect("generated list of paths");

    // TODO: Warn about paths that are neither linked, nor children of linked directories.

    // TODO: print outcome (no-op/dry-run/ok/partial/error).
    if matches.is_present("dry-run") {
        println!("Dry run: not applying any changes",);
    } else {
        for path in paths.iter_mut() {
            path.apply(&context);
        }
    };

    paths.sort();

    for path in paths.iter() {
        println!("{}", path);
    }
}
