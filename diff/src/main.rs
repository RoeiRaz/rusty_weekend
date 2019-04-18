/*
 * BSD 2-Clause License
 *
 * Copyright (c) @year, Roei Rosenzweig
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 *  Redistributions of source code must retain the above copyright notice, this
 *   list of conditions and the following disclaimer.
 *
 *  Redistributions in binary form must reproduce the above copyright notice,
 *   this list of conditions and the following disclaimer in the documentation
 *   and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

mod diff;
extern crate clap;

use diff::DiffScript;
use std::fs::File;
use std::io::{Lines, BufRead, BufReader, Write};
use clap::{App, Arg};


fn main() {
    let arg_matches;
    let app;
    let original_file;
    let target_file;
    let mut result_file;

    app = App::new("diff")
        .arg(Arg::with_name("original_file").short("o").required(true).index(1))
        .arg(Arg::with_name("target_file").short("t").required(true).index(2))
        .arg(Arg::with_name("result_file").short("r").required(false).default_value("diff"));

    arg_matches = app.get_matches();

    original_file = File::open(arg_matches.value_of("original_file").unwrap()).unwrap();
    target_file = File::open(arg_matches.value_of("target_file").unwrap()).unwrap();
    result_file = File::create(arg_matches.value_of("result_file").unwrap()).unwrap();

    let mut original: Vec<String> = BufReader::new(original_file).lines()
        .map(|x| x.unwrap()).collect();
    let mut target: Vec<String> = BufReader::new(target_file).lines()
        .map(|x| x.unwrap()).collect();

    result_file.write(serde_json::to_string(&diff::diff(original, target)).unwrap().as_bytes());
}