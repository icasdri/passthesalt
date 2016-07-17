/*
 * Copyright 2016 icasdri
 *
 * This file is part of passthesalt.
 * See COPYING for licensing details.
 */

use std::io::Write;
use std::io::BufWriter;
use std::io::stderr;
use std::io::stdin;
use std::fs::OpenOptions;
use std::process;

extern crate libpassthesalt as pts;
use pts::PtsError as PE;

#[macro_use]
extern crate clap;
use clap::ArgMatches;

mod args;

static E_STDERR: &'static str = "failed printing to stderr";
static E_STDIN: &'static str = "failed to read user input";
macro_rules! divider { () => ("--------------------------------------------------------") }
macro_rules! ef_filecreate { () => ("Failed to create file '{}'. The file may already exist or you may not have permission.") }
macro_rules! ef_filewrite { () => ("Failed to write to file '{}'. You may not have permission to write there.") }

enum MainError<'a> {
    UsageProblem(&'a ArgMatches<'a>, &'static str),
    InvalidInput,
    FileIo(String),
    Inner(PE)
}
use MainError as ME;

fn handle_key<'a>(m: &'a ArgMatches) -> Result<(), MainError<'a>> {
    try!(pts::init().map_err(|e| ME::Inner(e)));

    if m.is_present("new_key") {
        let (public_key_material, private_key_material) =
            try!(pts::new_keypair().map_err(|e| ME::Inner(e)));

        let mut buf = String::new();
        let priv_target_path = match m.value_of("private_key_output") {
            Some(f) => f,
            None => { // prompt user
                write!(stderr(), concat!(
                    "Enter file in which to save the private key\n",
                    "    (defaults to my_pts_key.txt): ")).expect(E_STDERR);
                stdin().read_line(&mut buf).expect(E_STDIN);
                write!(stderr(), "\n").expect(E_STDERR);
                let user_input = buf.trim();
                if user_input.len() > 0 {
                    user_input
                } else {
                    "my_pts_key.txt"
                }
            }
        };

        let priv_target_file = try!(OpenOptions::new().write(true).create_new(true)
            .open(priv_target_path)
            .map_err(|_| ME::FileIo(format!(ef_filecreate!(), priv_target_path))));

        let mut writer = BufWriter::new(priv_target_file);
        try!(writeln!(writer, "{}", private_key_material)
            .map_err(|_| ME::FileIo(format!(ef_filewrite!(), priv_target_path))));

        writeln!(stderr(), concat!(
            "Your private key has been saved to '{}'.\n",
            "Keep this file in a secure but accessible place.\n",
            "\n",
            "Below is your public key. It is a series of short words.\n",
            "Share it with people you would like to exchange messages with.\n",
            divider!()), priv_target_path).expect(E_STDERR);

        println!("{}", public_key_material);

        writeln!(stderr(), divider!()).expect(E_STDERR);

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
            },
            ME::FileIo(message) => {
                (format!("{}", message), 40)
            },
            _ => unimplemented!()
        };
        writeln!(stderr(), "{}", output).expect(E_STDERR);
        process::exit(exit_code);
    }
}
