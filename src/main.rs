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
    Generic(String),
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

    let public_key_string = match m.value_of("pub_key") {
        Some(p) => p.to_owned(),
        None => { // prompt user for public key
            write!(stderr(), concat!(
                "Enter the public key of the person you're exchanging\n",
                "messages with (they should've given this to you).\n",
                "> ")).expect(E_STDERR);

            let mut buf = String::new();
            stdin().read_line(&mut buf).expect(E_STDIN);
            write!(stderr(), "\n").expect(E_STDERR);
            buf.trim().to_owned()
        }
    };
    Ok((public_key_string, private_key_string))
}

fn read_input<'a>(m: &'a ArgMatches, prompt: &'static str) -> Result<Vec<u8>, MainError<'a>> {
    let mut target = Vec::new();
    if let Some(input_file_path) = m.value_of("input_file") {
        let mut input_file = try!(File::open(input_file_path)
            .or(Err(ME::FileIo(FI::Open, input_file_path.to_owned()))));
        let mut reader = BufReader::new(input_file);
        try!(reader.read_to_end(&mut target)
            .or(Err(ME::FileIo(FI::Read, input_file_path.to_owned()))));
    } else { // prompt user
        writeln!(stderr(), concat!(
                "{}. Press Enter then Ctrl+D when finished.\n",
                divider!()), prompt).expect(E_STDERR);
        stdin().read_to_end(&mut target).expect(E_STDIN);
        writeln!(stderr(), divider!()).expect(E_STDERR);
    }
    Ok(target)
}

fn handle_encrypt<'a>(m: &'a ArgMatches) -> Result<(), MainError<'a>> {
    try!(pts::init().map_err(|e| ME::Inner(e)));

    let (public_key_string, private_key_string) = try!(get_keys_from_args(m));
    let message_bytes = try!(read_input(m, "Write your message below"));

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

fn handle_decrypt<'a>(m: &'a ArgMatches) -> Result<(), MainError<'a>> {
    try!(pts::init().map_err(|e| ME::Inner(e)));

    let (public_key_string, private_key_string) = try!(get_keys_from_args(m));
    let input = try!(read_input(m, "Enter the encrypted content below"));

    let cipher_text = try!(std::str::from_utf8(&input)
                           .or(Err(ME::Inner(PE::DecryptParse))));

    let plain_bytes = try!(
        pts::decrypt(&public_key_string, &private_key_string, cipher_text)
            .map_err(|e| ME::Inner(e)));

    let mut buf = String::new();
    if let Some(output_file_path) = match m.value_of("output_file") {
        None => match std::str::from_utf8(&plain_bytes) {
            Ok(plain_text) => {
                println!("{}", plain_text);
                None
            },
            Err(_) => { // tell user that output cannot be printed
                write!(stderr(), concat!(
                        "The decrypted content is not text.\n",
                        "Please specify a file to save it to: ")).expect(E_STDERR);
                stdin().read_line(&mut buf).expect(E_STDIN);
                write!(stderr(), "\n").expect(E_STDERR);
                let user_input = buf.trim();
                if user_input.len() > 0 {
                    Some(user_input)
                } else {
                    return Err(ME::Generic("No file specified.".to_owned()));
                }
            }
        },
        Some(explicitly_given) => Some(explicitly_given)
    } {
        let output_file = try!(OpenOptions::new().write(true).create_new(true)
            .open(output_file_path)
            .or(Err(ME::FileIo(FI::Create, output_file_path.to_owned()))));

        let mut writer = BufWriter::new(output_file);
        try!(writeln!(writer, "{}", cipher_text)
            .or(Err(ME::FileIo(FI::Write, output_file_path.to_owned()))));

        writeln!(stderr(), "Your decrypted message/file has been saved to '{}'.",
            output_file_path).expect(E_STDERR);
    }

    Ok(())
}

fn main() {
    let matches = args::app().get_matches();

    let result = match matches.subcommand() {
        ("key", Some(m)) => handle_key(m),
        ("encrypt", Some(m)) => handle_encrypt(m),
        ("decrypt", Some(m)) => handle_decrypt(m),
        _ => unreachable!()
    };

    if let Err(err) = result {
        let (output, exit_code) = match err {
            ME::UsageProblem(u, message) => {
                (format!("{}\n\n{}\n\nFor more information try --help",
                    message, u.usage()), 1)
            },
            ME::Generic(message) => {
                (message, 7)
            },
            ME::Inner(e) => match e {
                PE::FatalInit | PE::FatalEncode => (format!("A fatal internal error has occurred: code {:?}.", e), 9),
                PE::PublicKeyParse => ("Failed to parse public key. Please make sure that you inputted the public key correctly (it is a series of 24 short words).".to_owned(), 11),
                PE::PublicKeyLength => ("Incorrect public key length. Please make sure that you inputted the public key correctly (it is a series of 24 short words).".to_owned(), 12),
                PE::PrivateKeyParse => ("Failed to parse private key. Please make sure that the file after -i or --me is indeed your private key file.".to_owned(), 13),
                PE::PrivateKeyLength => ("Incorrect private key length. Please make sure that the file after -i or --me is indeed your private key file.".to_owned(), 14),
                PE::DecryptParse => ("Failed to parse the encrypted input. Please make sure you copied/entered the input correctly. Otherwise, it may be corrupt.".to_owned(), 21),
                PE::DecryptPhase => (concat!("Decryption failed. Could not verify sender by public key. ",
                                             "Please make sure that you inputted the sender's public key correctly ",
                                             "(it is a series of 24 short words). Otherwise, the input may be corrupt, ",
                                             "or someone is trying to impersonate the sender.").to_owned(), 22),
                PE::DecryptLength => ("Incorrect decryption input length Please make sure you copied/entered the input correctly. Otherwise it may be corrupt".to_owned(), 23)
            },
            ME::FileIo(fi_type, file) => {
                (match fi_type {
                    FI::Create => format!("Failed to create file '{}'. The file may already exist or you may not have permission.", file),
                    FI::Open => format!("Failed to open file '{}' for reading. The file may not exist.", file),
                    FI::Read => format!("Failed to read file '{}'. The file may not be accessible.", file),
                    FI::Write => format!("Failed to write to file '{}'. You may not have permission to write there.", file)
                }, 4)
            },
        };
        writeln!(stderr(), "error: {}", output).expect(E_STDERR);
        process::exit(exit_code);
    }
}
