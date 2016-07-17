/*
 * Copyright 2016 icasdri
 *
 * This file is part of passthesalt.
 * See COPYING for licensing details.
 */

use std::io::Read;
use std::io::BufReader;
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
            .or(Err(ME::FileIo(FI::Create, priv_target_path.to_owned()))));

        let mut writer = BufWriter::new(priv_target_file);
        try!(writeln!(writer, "{}", private_key_material)
            .or(Err(ME::FileIo(FI::Write, priv_target_path.to_owned()))));

        writeln!(stderr(), concat!(
            "Your private key has been saved to '{}'.\n",
            "Keep this file in a secure but accessible place.\n",
            "\n",
            "Your public key is below. It is a series of short words.\n",
            "Share it with people you would like to exchange messages with.\n",
            divider!()), priv_target_path).expect(E_STDERR);

        println!("{}", public_key_material);

        writeln!(stderr(), divider!()).expect(E_STDERR);

        Ok(())
    } else {
        Err(ME::UsageProblem(m, concat!(
              "Please specify a flag to perform a key-related operation.\n",
              "       For instance, pass --new to generate a new key pair.")))
    }
}

fn get_keys_from_args<'a>(m: &'a ArgMatches) -> Result<(String, String), MainError<'a>> {
    let public_key_str = m.value_of("pub_key").unwrap(); // is required arg
    let private_key_path = m.value_of("priv_key").unwrap(); // is required arg

    let mut private_key_file = try!(File::open(private_key_path)
        .or(Err(ME::FileIo(FI::Open, private_key_path.to_owned()))));

    if let Ok(metadata) = private_key_file.metadata() {
        // sanity check on file size (if possible)
        if metadata.len() > 100 {
            return Err(ME::Inner(PE::PrivateKeyLength));
        }
    }

    let mut private_key_string = String::new();
    try!(private_key_file.read_to_string(&mut private_key_string)
        .or(Err(ME::FileIo(FI::Read, private_key_path.to_owned()))));

    Ok((public_key_str.to_owned(), private_key_string))
}

fn read_message<'a>(m: &'a ArgMatches) -> Result<Vec<u8>, MainError<'a>> {
    if let Some(input_file_path) = m.value_of("input_file") {
        let mut input_file = try!(File::open(input_file_path)
            .or(Err(ME::FileIo(FI::Open, input_file_path.to_owned()))));
        let mut reader = BufReader::new(input_file);
        let mut target = Vec::new();
        try!(reader.read_to_end(&mut target)
            .or(Err(ME::FileIo(FI::Read, input_file_path.to_owned()))));
        Ok(target)
    } else { // prompt user
        writeln!(stderr(), concat!(
                "Write your message below. Press Enter then Ctrl+D when finished.\n",
                divider!())).expect(E_STDERR);
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer).expect(E_STDIN);
        writeln!(stderr(), divider!()).expect(E_STDERR);
        Ok(buffer.into_bytes())
    }
}

fn handle_encrypt<'a>(m: &'a ArgMatches) -> Result<(), MainError<'a>> {
    try!(pts::init().map_err(|e| ME::Inner(e)));

    let (public_key_string, private_key_string) = try!(get_keys_from_args(m));
    let message_bytes = try!(read_message(m));

    let cipher_text = try!(
        pts::encrypt(&public_key_string, &private_key_string, &message_bytes)
            .map_err(|e| ME::Inner(e)));

    if let Some(output_file_path) = m.value_of("output_file") {
        let output_file = try!(OpenOptions::new().write(true).create_new(true)
            .open(output_file_path)
            .or(Err(ME::FileIo(FI::Create, output_file_path.to_owned()))));

        let mut writer = BufWriter::new(output_file);
        try!(writeln!(writer, "{}", cipher_text)
            .or(Err(ME::FileIo(FI::Write, output_file_path.to_owned()))));

        writeln!(stderr(), "Your encrypted message/file has been saved to '{}'.",
            output_file_path).expect(E_STDERR);
    } else { // print to stdout
        println!("{}", cipher_text);
    }
    Ok(())
}

fn handle_decrypt(m: &ArgMatches) {
    // pts::init();
}

fn main() {
    let matches = args::app().get_matches();

    let result = match matches.subcommand() {
        ("key", Some(m)) => handle_key(m),
        ("encrypt", Some(m)) => handle_encrypt(m),
        // ("decrypt", Some(m)) => handle_decrypt(m),
        _ => unreachable!()
    };

    if let Err(err) = result {
        let (output, exit_code) = match err {
            ME::UsageProblem(u, message) => {
                (format!("{}\n\n{}\n\nFor more information try --help",
                    message, u.usage()), 1)
            },
            ME::InvalidInput(message) => {
                (message, 2)
            },
            ME::Inner(ref e) if *e == PE::FatalInit || *e == PE::FatalEncode => {
                (format!("A fatal internal error has occurred: code {:?}", e), 102)
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
        writeln!(stderr(), "error: {}", output).expect(E_STDERR);
        process::exit(exit_code);
    }
}
