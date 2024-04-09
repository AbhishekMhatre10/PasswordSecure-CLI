#![deny(clippy::unwrap_used, clippy::expect_used)]

use anyhow::{Context, Result};
use clap::{Arg, Command};
use hashassin_core::{generate_passwords, hash_passwords};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use tracing::{debug, info, Level};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let mtch_com = Command::new("Hashassin")
        .subcommand(
            Command::new("gen-passwords")
                .arg(
                    clap::Arg::new("min-chars")
                        .long("min-chars")
                        .takes_value(true)
                        .default_value("4"),
                )
                .arg(
                    clap::Arg::new("max-chars")
                        .long("max-chars")
                        .takes_value(true)
                        .default_value("4"),
                )
                .arg(
                    clap::Arg::new("out-path")
                        .long("out-path")
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("threads")
                        .long("threads")
                        .takes_value(true)
                        .default_value("1"),
                )
                .arg(
                    clap::Arg::new("num-to-gen")
                        .long("num-to-gen")
                        .takes_value(true)
                        .default_value("10"),
                ),
        )
        .subcommand(
            Command::new("gen-hashes")
                .arg(
                    clap::Arg::new("in-path")
                        .long("in-path")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::new("out-path")
                        .long("out-path")
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("algorithm")
                        .long("algorithm")
                        .takes_value(true)
                        .default_value("SHA256"),
                )
                .arg(
                    Arg::new("threads")
                        .long("threads")
                        .takes_value(true)
                        .default_value("1"),
                ),
        )
        .get_matches();

    match mtch_com.subcommand() {
        Some(("gen-passwords", sub_m)) => gen_passwords_func(sub_m)?,
        Some(("gen-hashes", sub_m)) => gen_hashes_func(sub_m)?,
        _ => unreachable!(),
    }

    Ok(())
}

fn gen_passwords_func(sub_m: &clap::ArgMatches) -> Result<()> {
    let var_min_character = sub_m
        .value_of_t::<usize>("min-chars")
        .context("Invalid value for min-chars")?;
    let var_max_character = sub_m
        .value_of_t::<usize>("max-chars")
        .context("Invalid value for max-chars")?;
    let num_of_pwds = sub_m
        .value_of_t::<usize>("num-to-gen")
        .context("Invalid value for num-to-gen")?;
    let cpu_threads = sub_m
        .value_of_t::<usize>("threads")
        .context("Invalid value for threads")?;
    let file_outputpath = sub_m.value_of("out-path");

    if var_min_character == 0 {
        eprintln!("Minimum characters must be greater than zero.");
        std::process::exit(1);
    }

    if var_max_character == 0 {
        eprintln!("Maximum characters must be greater than zero.");
        std::process::exit(1);
    }

    if cpu_threads == 0 {
        eprintln!("Threads must be greater than zero.");
        std::process::exit(1);
    }

    if num_of_pwds == 0 {
        eprintln!("Number of passwords must be greater than zero.");
        std::process::exit(1);
    }

    if var_max_character < var_min_character {
        eprintln!("Maximum characters must be greater than or equal to minimum characters.");
        std::process::exit(1);
    }

    debug!(
        "Log > Generating passwords with min_chars: {}, max_chars: {}, count: {}, threads: {}",
        var_min_character, var_max_character, num_of_pwds, cpu_threads
    );
    let passwords = generate_passwords(
        num_of_pwds,
        var_min_character,
        var_max_character,
        cpu_threads,
    );

    match file_outputpath {
        Some(path) => {
            let pwds_file = File::create(path)?;
            let mut buffered_write_file = BufWriter::new(pwds_file);
            for pwd in passwords.iter() {
                writeln!(buffered_write_file, "{}", pwd)?;
            }
            info!(
                "Passwords successfully written to the following path: {}",
                path
            );
        }
        None => {
            passwords.iter().for_each(|pwd| println!("{}", pwd));
            info!("Passwords printed to stdout command line");
        }
    }

    Ok(())
}

fn gen_hashes_func(sub_m: &clap::ArgMatches) -> Result<()> {
    let file_inpath = Path::new(sub_m.value_of("in-path").context("Path is required")?);
    let file_outputpath = sub_m.value_of("out-path");
    let algorithm = sub_m.value_of("algorithm").unwrap_or("SHA256");

    let cpu_threads = sub_m
        .value_of_t::<usize>("threads")
        .context("Invalid value for threads")?;

    if cpu_threads == 0 {
        eprintln!("Threads must be greater than zero.");
        std::process::exit(1);
    }

    let pwds_file = File::open(file_inpath)?;
    let mut buffered_read_file = BufReader::new(pwds_file);
    let mut pwds_file_content = String::new();
    buffered_read_file.read_to_string(&mut pwds_file_content)?;

    let passwords: Vec<String> = pwds_file_content.lines().map(ToOwned::to_owned).collect();

    let hashed_passwords = hash_passwords(&passwords, algorithm, cpu_threads)?;

    if let Some(path) = file_outputpath {
        let hash_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(Path::new(path))?;
        let mut buffered_write_file = BufWriter::new(hash_file);
        for hash in hashed_passwords.iter() {
            writeln!(buffered_write_file, "{}", hash)?;
        }
        info!(
            "Password Hashes successfully written to the following path: {}",
            path
        );
    } else {
        for hash in hashed_passwords.iter() {
            println!("{}", hash);
        }
        info!("Password Hashes successfully printed to stdout command line");
    }

    Ok(())
}
