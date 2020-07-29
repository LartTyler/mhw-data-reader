use std::path::Path;
use std::process;

use clap::{App, AppSettings, Arg, ArgMatches, crate_version, SubCommand};

use mhw_data_reader::{gmd, itm};

fn main() {
    let matches = App::new("mhw-data-reader")
        .version(crate_version!())
        .author("Tyler Lartonoix <tyler@lartonoix.com>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("gmd")
                .about("Parses a GMD file and dumps it's output to stdout")
                .arg(
                    Arg::with_name("input")
                        .value_name("FILE")
                        .help("Specifies the file to load")
                        .takes_value(true)
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("itm")
                .about("Parses an ITM file and dumps it's output to stdout")
                .arg(
                    Arg::with_name("input")
                        .value_name("FILE")
                        .help("Specifies the file to load")
                        .takes_value(true)
                        .required(true)
                )
                .arg(
                    Arg::with_name("link")
                        .long("link")
                        .short("l")
                        .value_name("GMD_FILE")
                        .help("If specified, loads and links a GMD file (for strings)")
                        .takes_value(true)
                )
        )
        .get_matches();

    let (keys, values) = match matches.subcommand() {
        ("gmd", Some(args)) => parse_gmd(args),
        ("itm", Some(args)) => parse_itm(args),
        _ => unreachable!(),
    };

    for i in 0..keys.len() {
        println!("{:?} = {:?}", keys.get(i).unwrap(), values.get(i).unwrap());
    }
}

fn parse_gmd(matches: &ArgMatches) -> (Vec<Option<String>>, Vec<String>) {
    match gmd::parse(load_file(matches).as_ref()) {
        Ok(document) => {
            let mut keys = Vec::with_capacity(document.entries.len());
            let mut strings = Vec::with_capacity(document.entries.len());

            for entry in &document.entries {
                keys.push(entry.key.clone());
                strings.push(entry.value.clone())
            }

            (keys, strings)
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn parse_itm(matches: &ArgMatches) -> (Vec<Option<String>>, Vec<String>) {
    match itm::parse(load_file(matches).as_ref()) {
        Ok(mut document) => {
            if let Some(path) = matches.value_of("link") {
                match gmd::parse(load_file_from_path(path).as_ref()) {
                    Ok(gmd_document) => {
                        document.import_gmd(&gmd_document);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                }
            }

            let mut keys = Vec::with_capacity(document.entries.len());
            let mut values = Vec::with_capacity(document.entries.len());

            for entry in &document.entries {
                keys.push(Some(format!("ITEM_{:05}", entry.id)));
                values.push(format!("{:?}", entry));
            }

            (keys, values)
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn load_file(matches: &ArgMatches) -> Vec<u8> {
    load_file_from_path(matches.value_of("input").unwrap())
}

fn load_file_from_path(path: &str) -> Vec<u8> {
    let path = Path::new(path);

    if !path.is_file() {
        eprintln!("Input file could not be read");
        process::exit(1);
    }

    match std::fs::read(path) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
