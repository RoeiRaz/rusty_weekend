mod diff;
extern crate clap;

use diff::DiffScript;
use std::fs::{File, OpenOptions};
use std::io::{Lines, BufRead, BufReader, Write};
use clap::{App, Arg, ArgGroup, ArgMatches};
use std::process::exit;
use serde::Deserialize;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

fn subcommand_compute(arg_matches: &ArgMatches) {
    let original_file;
    let target_file;
    let diff_script_encoding;

    original_file = File::open(arg_matches.value_of("original_file").unwrap()).unwrap();
    target_file = File::open(arg_matches.value_of("target_file").unwrap()).unwrap();

    let mut original: Vec<String> = BufReader::new(original_file).lines()
        .map(|x| x.unwrap()).collect();
    let mut target: Vec<String> = BufReader::new(target_file).lines()
        .map(|x| x.unwrap()).collect();

    diff_script_encoding = serde_json::to_string(&diff::diff(original, target)).unwrap();

    match arg_matches.value_of("patch_file") {
        Some(p) => {File::create(p).unwrap().write(diff_script_encoding.as_bytes());},
        _ => println!("{}", diff_script_encoding)
    };
}

fn subcommand_patch(arg_matches: &ArgMatches) {
    let original_file = File::open(arg_matches.value_of("original_file").unwrap()).unwrap();
    let patch_file = File::open(arg_matches.value_of("patch_file").unwrap()).unwrap();
    let mut result_file = File::create(arg_matches.value_of("result_file")
        .unwrap_or(&arg_matches.value_of("original_file").unwrap())).unwrap();
    let reader = BufReader::new(patch_file);
    let diff_script: DiffScript<String> = serde_json::from_reader(reader).unwrap();
    let original_lines: Vec<String> = BufReader::new(original_file).lines()
        .map(|x| x.unwrap()).collect();
    let patched_content = diff_script.apply_copy(&original_lines).join(LINE_ENDING);
    result_file.write(patched_content.as_bytes());
}

fn main() {
    let arg_matches;
    let mut app: App;
    let mut merge_subcmd: App;
    let mut diff_subcmd: App;
    let mut patch_subcmd: App;

    diff_subcmd = App::new("compute")
        .arg(Arg::with_name("original_file").short("o").required(true).index(1))
        .arg(Arg::with_name("target_file").short("t").required(true).index(2))
        .arg(Arg::with_name("patch_file").short("p").required(false));

    patch_subcmd = App::new("patch")
        .arg(Arg::with_name("original_file").short("o").required(true).index(1))
        .arg(Arg::with_name("patch_file").short("p").required(true).index(2).default_value("diff"))
        .arg(Arg::with_name("result_file").short("p").required(false).index(3));

    app = App::new("diff").version("0.0.1-ghostly")
        .subcommand(diff_subcmd).subcommand(patch_subcmd);

    arg_matches = app.get_matches();

    match arg_matches.subcommand() {
        ("compute", Some(arg_matches)) => subcommand_compute(arg_matches),
        ("patch", Some(arg_matches)) => subcommand_patch(arg_matches),
        _ => {println!("{}", arg_matches.usage()); exit(1);}
    };

}