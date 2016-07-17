/*
 * Copyright 2016 icasdri
 *
 * This file is part of passthesalt.
 * See COPYING for licensing details.
 */

use std::io::Write;
use std::process;

extern crate libpassthesalt as pts;
use pts::PtsError as PE;

#[macro_use]
extern crate clap;
use clap::ArgMatches;

mod args;

enum MainError<'a> {
    UsageProblem(&'a ArgMatches<'a>, &'static str),
    InvalidInput,
    Inner(PE)
}
use MainError as ME;

fn handle_key<'a>(m: &'a ArgMatches) -> Result<(), MainError<'a>> {
    try!(pts::init().map_err(|e| MainError::Inner(e)));

    if m.is_present("new_key") {
        Ok(())
    } else {
        Err(ME::UsageProblem(m, concat!(
              "error: Please specify a flag to perform a key-related operation.\n",
              "       For instance, pass --new to generate a new key pair.")))
    }
}

fn handle_encrypt(m: &ArgMatches) {
    // pts::init();
}

fn handle_decrypt(m: &ArgMatches) {
    // pts::init();
}

fn main() {
    let mut stderr = std::io::stderr();
    let matches = args::app().get_matches();

    let result = match matches.subcommand() {
        ("key", Some(m)) => handle_key(m),
        // ("encrypt", Some(m)) => handle_encrypt(m),
        // ("decrypt", Some(m)) => handle_decrypt(m),
        _ => unreachable!()
    };

    if let Err(err) = result {
        let (output, exit_code) = match err {
            ME::UsageProblem(u, message) => {
                (format!("{}\n\n{}\n\nFor more information try --help",
                    message, u.usage()), 1)
            },
            ME::Inner(PE::Fatal(message)) => {
                (format!("{}", message), 102)
            }
            _ => unimplemented!()
        };
        writeln!(stderr, "{}", output).expect("failed printing to stderr");
        process::exit(exit_code);
    }
}
