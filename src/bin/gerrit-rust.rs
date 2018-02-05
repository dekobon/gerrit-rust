
//! main entry for `gerrit-rust`

extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate libgerrit;
extern crate git2;
extern crate gron;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rustc_serialize;
extern crate serde_json;
extern crate toml_config;
extern crate url;
extern crate netrc;

pub mod changes;
pub mod config;
pub mod topic;
pub mod gerritapi;

use clap::{Arg, App};
use libgerrit::error::GGRError;
use std::error::Error;
use std::io::Write;
use std::process::exit;

mod version {
    // VERSION and VERSION_CSTR are generated by build.rs
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}
pub use version::VERSION;
pub use version::VERSION_CSTR;

fn init_log() {
    let format = |formatter: &mut env_logger::fmt::Formatter, record: &log::Record| {
        writeln!(formatter, "[{:5.5}] [{}] [{}] - {}", record.level(),
                chrono::Local::now().to_rfc3339(),
                record.module_path().unwrap_or("no module_path"),
                record.args())
    };

    let mut builder = env_logger::Builder::new();
    builder.format(format).filter(None, log::LevelFilter::Info);

    if let Ok(ref rl) = std::env::var("RUST_LOG") {
        builder.parse(rl);
    }

    let _ = builder.init();
}

fn main() {
    init_log();

    let mut app = App::new("gerrit-rust")
        .author("Silvio Fricke <silvio.fricke@gmail.com>")
        .version(VERSION)
        .about("some gerrit tools")
        .arg(Arg::with_name("dry-run")
             .long("dry-run")
             .help("Blaming what will be done, but does nothing")
         )
        .subcommand(topic::menu())
        .subcommand(changes::menu())
        .subcommand(config::menu())
        .subcommand(gerritapi::menu())
        ;

    let matches = app.clone().get_matches();

    let configfile = match config::ConfigFile::discover(".", ".ggr.conf") {
        Ok(c) => c,
        Err(x) => {
            println!("Problem with loading of config file:");
            println!("{}", x.to_string());
            exit(-1);
        },
    };
    let mut config = config::Config::from_configfile(configfile);
    if ! config.is_valid() {
        panic!(GGRError::General("problem with configfile".to_string()));
    }

    config.set_dry_run(matches.is_present("dry-run"));

    let out = match matches.subcommand() {
        ("topic", Some(x)) => { topic::manage(x, &config) },
        ("changes", Some(x)) => { changes::manage(x, &config) },
        ("config", Some(x)) => { config::manage(x) },
        ("gerritapi", Some(x)) => { gerritapi::manage(x, &config) },
        _ => { let _ = app.print_help(); Ok(()) },
    };

    if let Err(e) = out {
        debug!("{:?}", e);
        println!("Error: {}", e);
        if let Some(cause) = e.cause() {
            println!("-> {}", cause);
        }
    };
}
