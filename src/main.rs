use rpassword::prompt_password;
use scrypt::{scrypt, Params};
use std::env::{args, var};
use std::fs::{read_to_string, OpenOptions};
use std::io::{stdin, stdout, BufRead, ErrorKind, IsTerminal, Write};

mod wordlist;
use wordlist::WORDLIST;

const XKCDGET_VERSION: &str = "2.1.1"; // semantic versioning!
const WORDLIST_LEN: usize = 2048;
const KEY_LEN: usize = 32;
const AMOUNT_WORDS: u8 = 4;
const DEFAULT_PIN_LEN: u8 = 4;

/// Return path to revocation file
fn get_revocation_filename() -> String {
    let homedir = var("HOME").expect("HOME environment variable unset or invalid");
    format!("{}/.xkcdget-revocation", homedir)
}

/// Calculate the hash used for revocation.
fn get_revocation_hash(key: &[u8; KEY_LEN]) -> String {
    sha256::digest(key)
}

/// calculate and print password entropy
fn print_entropy() {
    let bits_per_word = (WORDLIST_LEN as f32).log2();
    eprintln!(
        "Entropy: {} bits ({} bits per word)",
        bits_per_word * AMOUNT_WORDS as f32,
        bits_per_word
    );
}

/// Interactively ask for a domain and return it.
fn get_domain() -> String {
    let stdin = stdin();
    let mut domain = String::new();

    // Ask for interactive domain input if we're on a terminal
    if stdin.is_terminal() {
        print!("Domain: ");
        stdout().flush().expect("Can't flush stdout");
    }

    // Read domain, remove newline and return
    stdin
        .lock()
        .read_line(&mut domain)
        .expect("Expecting argument: Domain");
    String::from(domain.trim())
}

/// Read hashes of passwords that have been revoked.
fn get_revocation_hashes() -> Vec<String> {
    // read file
    let revocation_filename = get_revocation_filename();
    let file_content = read_to_string(&revocation_filename).unwrap_or_else(|e| {
        // treat nonexisting file like an empty file
        if e.kind() == ErrorKind::NotFound {
            String::new()
        } else {
            panic!("Error opening file {}: {}", revocation_filename, e)
        }
    });

    // return iterator over password hashes
    file_content
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Get the master password via invisible interactive input or from stdin.
fn get_master_password() -> String {
    // Get master password
    if stdin().is_terminal() {
        // If we're on a terminal, prompt the user
        prompt_password("Master password: ").expect("Could not read master password from terminal")
    } else {
        // If we're not on a terminal, read from stdin
        let mut buf = String::with_capacity(64);
        stdin()
            .lock()
            .read_line(&mut buf)
            .expect("We're not on a terminal, but no password was provided on stdin");
        String::from(buf.trim())
    }
}

/// Query for the master password and calculate salted hash of it and the domain.
fn get_key(domain: String) -> [u8; KEY_LEN] {
    let master_password = get_master_password();

    // generate key until one is found that has not been revoked
    let mut key = [0; KEY_LEN];
    // TODO 17 is the recommended CPU cost factor as of writing this code.
    // TODO test performance and also test performance of factor 18.
    // TODO Also check parameters for memory usage.
    let (log_n, r, p) = (17, 8, 16);
    let scrypt_params = Params::new(log_n, r, p, KEY_LEN).expect("Cannot create scrypt parameters");
    let revocation_hashes = get_revocation_hashes();
    for iteration in 0.. {
        // get password for this iteration
        let salt = format!("{}:{}", domain, iteration);
        scrypt(
            master_password.as_bytes(),
            salt.as_bytes(),
            &scrypt_params,
            &mut key,
        )
        .expect("scrypt failed");

        // if the key has been revoked do another round, else return it
        let revocation_hash = get_revocation_hash(&key);
        if revocation_hashes.contains(&revocation_hash) {
            eprintln!("Was revoked: {}", revocation_hash);
        } else {
            return key;
        }
    }

    unreachable!("The unconditional loop above must return the first non-revoked key");
}

/// Generate and print xkcdget password.
fn xkcdget(domain: String) -> String {
    // assert word list length so that we don't forget to change this code when
    // word list length changes.
    assert!(WORDLIST.len() == WORDLIST_LEN);

    // get password bits
    let key: [u8; KEY_LEN] = get_key(domain);

    // choose words
    // FIXME in 3.0
    let mut words = Vec::new();
    for word_i in 0..AMOUNT_WORDS {
        let offset = word_i as usize * 8;

        // FIXME: Use entropy optimally - see branch optimal-entropy-usage!
        let word_key: &[u8] = &key[offset..(offset + 8)];
        assert!(word_key.len() == 8); // key:[u8;8] would be the bigger hassle

        // read decoded bytes into u64
        let int_key: u64 =
            u64::from_le_bytes(word_key.try_into().expect("Cannot convert slice to u64"));

        // choose word
        let index = (int_key as usize) % WORDLIST_LEN;
        let word_uncap = WORDLIST[index];

        // capitalize word
        let mut word_chars = word_uncap.chars();
        let first_char = word_chars.next().expect("Chosen word is empty");
        let rest_chars = word_chars.flat_map(|c| c.to_lowercase());
        let word: String = first_char.to_uppercase().chain(rest_chars).collect();

        // add word
        words.push(word);
    }

    // print final password
    let words = words.join("");
    format!("{words}_1")
}

/// Generate and print xkcdget pin.
fn pin(domain: String, digits: u8) {
    let key = get_key(domain);
    todo!("choose digits");
}

/// Generate and revoke a password
fn revoke(domain: String) {
    let key = get_key(domain);
    let pw_revocation_hash = get_revocation_hash(&key);
    eprintln!("Revoking hash:{}", pw_revocation_hash);

    // add hash to revocation file
    let revocation_filename = get_revocation_filename();
    let mut revocation_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(revocation_filename)
        .expect("Cannot create/open revocation file name for appending");
    let _bytes_written = revocation_file
        .write(format!("{pw_revocation_hash}\n").as_bytes())
        .expect("Can't append revocation hash to revocation file");
    revocation_file
        .flush()
        .expect("Can't flush revocation file handle");
}

/// Dispatch according to program arguments.
fn main() {
    eprintln!("xkcdget {XKCDGET_VERSION}");
    print_entropy();
    let mut args = args();
    match args.nth(1) {
        // no argument = interactive mode
        None => println!("{}", xkcdget(get_domain())),

        // the first argument is either an action flag or a domain
        Some(arg) => match arg.as_str() {
            // known action flags
            "-r" | "--revoke" => revoke(args.next().unwrap_or_else(get_domain)),
            "-p" | "--pin" => pin(
                args.next().unwrap_or_else(get_domain),
                // get possibly supplied number argument for pin length
                args.next()
                    .map(|pinlen_arg| {
                        // check that argument is a number
                        pinlen_arg
                            .parse()
                            .unwrap_or_else(|_| panic!("Argument is not a number: {}", pinlen_arg))
                    })
                    .unwrap_or(DEFAULT_PIN_LEN),
            ),

            // not a known action flag, so treat as a domain
            _ => println!("{}", xkcdget(arg)),
        },
    }
}
