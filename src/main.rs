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
use std::fs::File;
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

enum FileIoErrorType {
    Create, Open, Read, Write
}
use FileIoErrorType as FI;

enum MainError<'a> {
    UsageProblem(&'a ArgMatches<'a>, &'static str),
    InvalidInput(String),
    FileIo(FileIoErrorType, String),
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
            .map_err(|_| ME::FileIo(FI::Create, priv_target_path.to_owned())));

        let mut writer = BufWriter::new(priv_target_file);
        try!(writeln!(writer, "{}", private_key_material)
            .map_err(|_| ME::FileIo(FI::Write, priv_target_path.to_owned())));

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

fn handle_encrypt<'a>(m: &'a ArgMatches) -> Result<(), MainError<'a>> {
    try!(pts::init().map_err(|e| ME::Inner(e)));

    let priv_key_path = m.value_of("priv_key").unwrap(); // is required arg
    let pub_key_str = m.value_of("pub_key").unwrap(); // is required arg

    let priv_key_file = try!(File::open(priv_key_path)
                             .map_err(|_| ME::FileIo(FI::Open, priv_key_path.to_owned())));
    // TODO: Testing only
    Ok(())
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
            ME::FileIo(fi_type, file) => {
                (match fi_type {
                    FI::Create => format!("Failed to create file '{}'. The file may already exist or you may not have permission.", file),
                    FI::Open => format!("Failed to open file '{}' for reading. The file may not exist.", file),
                    FI::Read => format!("Failed to read file '{}'. The file may not be accessible.", file),
                    FI::Write => format!("Failed to write to file '{}'. You may not have permission to write there.", file)
                }, 40)
            },
            _ => unimplemented!()
        };
        writeln!(stderr(), "{}", output).expect(E_STDERR);
        process::exit(exit_code);
    }
}
