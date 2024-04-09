#![deny(clippy::unwrap_used, clippy::expect_used)]
use rand::{prelude::SliceRandom, thread_rng, Rng};
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use thiserror::Error;

use tracing::debug;

#[derive(Error, Debug)]
pub enum HashError {
    #[error("Hashing error: {0}")]
    HashingError(String),
}

pub fn generate_passwords(
    num_of_pwds: usize,
    var_min_character: usize,
    var_max_character: usize,
    threads: usize,
) -> Vec<String> {
    let passwords = Arc::new(Mutex::new(Vec::with_capacity(num_of_pwds)));

    let var_pass_count = num_of_pwds / threads;
    let mut var_remain = num_of_pwds % threads;
    let set_printable_ascii_chars: Vec<char> = (32..=126).map(|c| c as u8 as char).collect(); //This is a set of valid ASCII
    debug!("Starting password generation with {} threads.", threads);
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let passwords_clone = Arc::clone(&passwords);
            let set_printable_ascii_chars_clone = set_printable_ascii_chars.clone();
            let thread_extra = if var_remain > 0 {
                var_remain -= 1;
                1
            } else {
                0
            };

            thread::spawn(move || {
                let mut var_randg = thread_rng();
                let count = var_pass_count + thread_extra;
                let local_passwords: Vec<String> = (0..count)
                    .map(|_| {
                        let length = var_randg.gen_range(var_min_character..=var_max_character);
                        (0..length)
                            .map(|_| {
                                *set_printable_ascii_chars_clone
                                    .choose(&mut var_randg)
                                    .unwrap_or(&'!')
                            })
                            .collect()
                    })
                    .collect();

                let mut pwds = match passwords_clone.lock() {
                    Ok(pwd) => pwd,
                    Err(err) => panic!("Error acquiring lock: {}", err),
                };
                pwds.extend(local_passwords);
            })
        })
        .collect();

    for handle in handles {
        match handle.join() {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error joining thread: {:#?}", err);
            }
        }
    }

    let locked_passwords = match passwords.lock() {
        Ok(locked) => locked,
        Err(err) => {
            panic!("Failed to acquire lock on passwords: {:?}", err);
            // Handle the error in some appropriate way
        }
    };
    locked_passwords.clone()
}

pub fn hash_passwords(
    input: &[String],
    algorithm: &str,
    threads: usize,
) -> Result<Vec<String>, HashError> {
    debug!(
        "Hashing passwords using {} algorithm across {} threads.",
        algorithm, threads
    );
    let (sender, receiver) = channel();
    let input = Arc::new(input.to_vec());
    let algorithm = algorithm.to_string(); // Clone the algorithm into a String

    let chunk_size = if input.len() % threads == 0 {
        input.len() / threads
    } else {
        input.len() / threads + 1
    };

    debug!("Starting hash generation with {} threads.", threads);
    for i in 0..threads {
        let input_clone = Arc::clone(&input);
        let sender_clone = sender.clone();
        let algorithm_clone = algorithm.clone(); // Clone for each thread

        thread::spawn(move || {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, input_clone.len());
            let hashes: Vec<(usize, String)> = input_clone[start..end]
                .iter()
                .enumerate()
                .map(|(index, password)| {
                    let hash = match algorithm_clone.as_str() {
                        "SHA256" => {
                            let mut hasher = Sha256::new();
                            hasher.update(password.as_bytes());
                            format!("{:x}", hasher.finalize())
                        }
                        "SHA512" => {
                            let mut hasher = Sha512::new();
                            hasher.update(password.as_bytes());
                            format!("{:x}", hasher.finalize())
                        }
                        "SHA224" => {
                            let mut hasher = Sha224::new();
                            hasher.update(password.as_bytes());
                            format!("{:x}", hasher.finalize())
                        }
                        "SHA384" => {
                            let mut hasher = Sha384::new();
                            hasher.update(password.as_bytes());
                            format!("{:x}", hasher.finalize())
                        }
                        _ => panic!("Unsupported algorithm"),
                    };
                    (start + index, hash)
                })
                .collect();
            match sender_clone.send(hashes) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error sending hash data: {:?}", err);
                }
            }
        });
    }

    let mut result: Vec<(usize, String)> = Vec::with_capacity(input.len());
    for _ in 0..threads {
        match receiver.recv() {
            Ok(partial) => {
                let mut partial_result = partial;
                result.append(&mut partial_result);
            }
            Err(err) => {
                eprintln!("Error receiving hash data : {:?}", err);
            }
        }
    }

    result.sort_by_key(|k| k.0);
    Ok(result.into_iter().map(|(_, hash)| hash).collect())
}
