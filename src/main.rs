extern crate torchbear_lib;
extern crate clap;
extern crate log;

use clap::{Arg, App as ClapApp, SubCommand};

fn main() {
    let matches = ClapApp::new("actix-lua-web")
        .version("0.1")
        .author("Kevin K. <kbknapp@gmail.com>")
        .about("Does awesome things")
                .arg(Arg::with_name("log")
           .long("log")
           .value_name("LEVEL")
           .help("Prints messages with log level <LEVEL>")
           .default_value("info")
           .takes_value(true))
        .arg(Arg::with_name("log scope")
           .long("log-scope")
           .value_name("SCOPE")
           .help("Wether to log everything in the dependency tree")
           .default_value("torchbear")
           .takes_value(true))
        .get_matches();

    let log_level = match matches.value_of("log").unwrap() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        l => {
              println!("{} is not a valid log level, available levels are:\n\terror, warn, info, debug or trace", l);
            std::process::exit(1)
        }
    };

    let log_everything = match matches.value_of("log scope").unwrap() {
          "torchbear" => false,
        "everything" => true,
        l => {
              println!("{} is not a valid log scope, available levels are 'torchbear' and 'everything'", l);
            std::process::exit(1)
        }
    };

    torchbear_lib::start(torchbear_lib::logger::Settings{
        level: log_level,
      everything: log_everything,
    });
}
