/*
 * Copyright 2016 icasdri
 *
 * This file is part of passthesalt.
 * See COPYING for licensing details.
 */

use clap::{Arg, App, SubCommand, AppSettings, ArgMatches};

pub fn app<'a, 'b>() -> App<'a, 'b> {
    let priv_key_arg =
        Arg::with_name("priv_key")
            .short("i")
            .long("identity")
            .long("me")
            .value_name("keyfile")
            .takes_value(true)
            .help("Your private key file");

    let pub_key_arg =
        Arg::with_name("pub_key")
            .short("t")
            .long("them")
            .value_name("key")
            .takes_value(true)
            .help("Their public key (directly, from a file, or as a contact name)");

    let contacts_file_arg =
        Arg::with_name("contacts_file")
            .short("c")
            .long("contacts")
            .value_name("file")
            .takes_value(true)
            .help("The contacts file to look up contacts from");

    let input_file_arg =
        Arg::with_name("input_file")
            .value_name("INPUT_FILE")
            .takes_value(true)
            .help("Input file to encrypt/decrypt (leave off to read from stdin)");

    let output_file_arg =
        Arg::with_name("output_file")
            .short("o")
            .long("output-file")
            .value_name("FILE")
            .takes_value(true)
            .help("Output file (leave off to print to stdout)");

    let key_subcommand =
        SubCommand::with_name("key")
            .arg(Arg::with_name("new_key")
                .short("n")
                .long("new")
                .help("Generate new public/private key pair"))
            .arg(Arg::with_name("private_key_output")
                .short("o")
                .long("output-private")
                .requires("new_key")
                .value_name("FILE")
                .takes_value(true)
                .help("The file to output the new private key to (prompts by default)"))
            .about("Generate new keys and other key-related operations");

    let encrypt_subcommand =
        SubCommand::with_name("encrypt")
            .arg(priv_key_arg.clone().required(true))
            .arg(pub_key_arg.clone().required(true))
            .arg(input_file_arg.clone())
            .arg(output_file_arg.clone())
            // .arg(contacts_file_arg.clone())
            .about("Encrypt a message to a recipient's public key");

    let decrypt_subcommand =
        SubCommand::with_name("decrypt")
            .arg(priv_key_arg.clone().required(true))
            .arg(pub_key_arg.clone().required(true))
            .arg(input_file_arg.clone())
            .arg(output_file_arg.clone())
            // .arg(contacts_file_arg.clone())
            .about("Decrypt a message from a sender's public key");

    App::new("passthesalt")
        .version(crate_version!())
        .global_setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(key_subcommand.display_order(1))
        .subcommand(encrypt_subcommand.display_order(2))
        .subcommand(decrypt_subcommand.display_order(3))
}
