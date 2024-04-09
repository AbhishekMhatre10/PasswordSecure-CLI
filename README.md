#Commands for executing Project

#We have performed cargo fmt, cargo check, cargo clippy, and cargo build and it did not give us any errors or warnings.

=> For generating passwords 
> cargo run gen-passwords --min-chars 3 --max-chars 8 --num-to-gen 9 --threads 4 --out-path passwords.txt

=> For generating Hashes for generated password
> cargo run gen-hashes --threads 4 --algorithm SHA512 --in-path passwords.txt --out-path hashes.txt 

=> For generating passwords (default values)
>  cargo run gen-passwords

**************************************************************************************


#Crates used for Project from blessed.rs

Crates used in hashassin:

anyhow: Error handling
clap: Command-line argument parsing
hashassin_core: Password generation and hashing
tracing: Logging
tracing-subscriber

Crates used in hashassin_core:

rand: Random number generation
sha2: SHA-256 hashing
thiserror: Custom error types
tracing: Logging

**************************************************************************************

=> Main.rs code explanation and work-flow

basic functionality of Main.rs is to take cammand-line input for generating password and generating hashes.

After adding provided instruction in project description , we have added "#![deny(clippy::unwrap_used, clippy::expect_used)]" to remove detect wheather our code is using unwrap and expect or not , on which running  "cargo clippy" on command line has given us no errors.

In main function serves as the entry point of the program. It initializes the logging framework provided by the tracing crate and parses command-line arguments using clap. It then matches the subcommands provided by the user (gen-passwords or gen-hashes) and calls respective functions accordingly.

gen-passwords: This subcommand generates random passwords. It parses command-line arguments such as minimum and maximum characters for passwords, the number of passwords to generate, and the number of threads to use. It then validates the input parameters and generates passwords using the generate_passwords_cli() function. Finally, it writes the generated passwords to a file or prints them to stdout. If values not provided with subcommand it automatically goes to default values set for minimum and maximum characters for passwords, the number of passwords to generate, and the number of threads to use.

gen-hashes: This subcommand generates hashes from passwords provided in a file. It parses command-line arguments such as input and output paths, hashing algorithm, and the number of threads to use. It validates input parameters, reads passwords from a file, generates hashes using the hash_passwords_cli() function, and writes the hashes to a file or prints them to stdout.

Error Handling: Error handling is performed using the anyhow crate, which provides convenient error handling utilities like the Context trait for adding context to errors. It ensures proper error messages are displayed if invalid input or file operations fail.

generate_passwords_cli(): This function generates random passwords based on provided parameters and writes them to a file or prints them to stdout.

generate_hashes_cli(): This function generates hashes from passwords read from a file and writes them to a file or prints them to stdout.

we have achieved concurrency in code by utilizing multi-threading for password generation and hashing to improve performance. It uses Rust's standard library features like std::thread and std::sync::mpsc for inter-thread communication.

The code incorporates logging using the tracing crate, allowing for flexible and structured logging statements with various log levels.


**************************************************************************************

=> lib.rs code explanation and work-flow

basic functionality of lib.rs is generating random passwords and hashing those passwords using multiple threads.

After adding provided instruction in project description , we have added "#![deny(clippy::unwrap_used, clippy::expect_used)]" to remove detect wheather our code is using unwrap and expect or not , on which running  "cargo clippy" on command line has given us no errors.

Two custom error types are defined using the thiserror crate: PasswordError for errors related to password generation and HashError for errors related to password hashing.

The public generate_passwords() function takes parameters such as the number of passwords to generate, minimum and maximum characters, and the number of threads to use. It utilizes Rust's standard library features for multi-threading (std::thread) and concurrent data access (std::sync::Arc and std::sync::Mutex) to generate passwords efficiently across multiple threads.

The public hash_passwords() function takes input passwords, hashing algorithm, and the number of threads to use.It utilizes channels (std::sync::mpsc::channel) for communication between threads and Rust's standard library features for multi-threading to hash passwords concurrently across multiple threads.Each thread hashes a portion of the input passwords using the specified algorithm (currently supports only SHA-256) and sends the hashed values through a channel.The main thread receives the hashed values, combines them into a single vector, sorts them, and returns the result.

Both the password generation and hashing functions utilize error handling mechanisms provided by the thiserror crate, allowing for custom error types and error messages.
Errors are propagated using Result types to provide detailed error information to the caller.

Logging statements are included using the tracing crate, which allows for structured logging with different log levels (debug in this case) to provide insights into the execution flow and any potential issues.