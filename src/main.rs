use rpassword::prompt_password;
use scrypt::{scrypt, Params};
use std::env::{args, var};
use std::fs::{read_to_string, OpenOptions};
use std::io::{stdin, stdout, BufRead, ErrorKind, IsTerminal, Write};

mod wordlists;
use wordlists::{ADJECTIVESLIST, NOUNSLIST, VERBSLIST};

const XKCDGET_VERSION: &str = "1.1.1"; // semantic versioning!
const NOUNSLIST_LEN: usize = 1525;
const ADJECTIVESLIST_LEN: usize = 528;
const VERBSLIST_LEN: usize = 1011;
const KEY_LEN: usize = 32;

/// Return path to revocation file
fn get_revocation_filename() -> String {
    let homedir = var("HOME").expect("HOME environment variable unset or invalid");
    format!("{}/.pwget2-revocation", homedir)
}

/// Calculate the hash used for revocation.
fn get_revocation_hash(password_str: &String) -> String {
    let hash = hex::decode(sha256::digest(password_str)).expect("Cannot hex-decode passwordStr");
    z85::encode(hash)
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
fn get_revoked_pw_hashes() -> Vec<String> {
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
fn get_scrypt_z85(domain: String) -> String {
    let master_password = get_master_password();

    // hash password until one is found that has not been revoked
    let mut password = [0; KEY_LEN];
    let (log_n, r, p) = (16, 8, 16);
    let scrypt_params = Params::new(log_n, r, p, KEY_LEN).expect("Cannot create scrypt parameters");
    let revoked_pw_hashes: Vec<String> = get_revoked_pw_hashes();
    for iteration in 0.. {
        // get password for this iteration
        let salt = format!("{}:{}", iteration, domain);
        scrypt(
            master_password.as_bytes(),
            salt.as_bytes(),
            &scrypt_params,
            &mut password,
        )
        .unwrap();
        let password_str = z85::encode(password);

        // if the password has been revoked do another round, else return it
        let pw_revocation_hash = get_revocation_hash(&password_str);
        if revoked_pw_hashes.contains(&pw_revocation_hash) {
            eprintln!("hash:{} is revoked", pw_revocation_hash);
        } else {
            return password_str;
        }
    }

    unreachable!("The unconditional loop above must return the first non-revoked password hash");
}

fn choose_word_from_list(wordlist: &[&str], seed: &str) -> String {
    assert!(seed.len() == 10);

    // z85 consumes 5 bytes at a time and decodes them into 4 bytes (32 bits).
    // decode 64 bits
    let key = z85::decode(seed).expect("Can't z85-decode password_str");

    // read decoded bytes into u64
    let int_key: u64 = u64::from_le_bytes(
        key.as_slice()[0..8]
            .try_into()
            .expect("Cannot convert slice to u64"),
    );

    // choose word
    let index = (int_key as usize) % wordlist.len();
    let word_uncap = wordlist[index];

    // capitalize word
    let mut word_chars = word_uncap.chars();
    let first_char = word_chars.next().expect("Chosen word is empty");
    let rest_chars = word_chars.flat_map(|c| c.to_lowercase());

    // return word
    first_char.to_uppercase().chain(rest_chars).collect()
}

/// Generate and print xkcdget password.
fn xkcdget(domain: String) -> String {
    // assert word list lengths so that we don't forget to change this code when
    // word list length changes.
    assert!(NOUNSLIST.len() == NOUNSLIST_LEN);
    assert!(ADJECTIVESLIST.len() == ADJECTIVESLIST_LEN);
    assert!(VERBSLIST.len() == VERBSLIST_LEN);

    // get password bits
    let password_str = get_scrypt_z85(domain);

    // choose words and return password
    format!(
        "{}{}{}{}_1",
        choose_word_from_list(&ADJECTIVESLIST, &(password_str[0..10])),
        choose_word_from_list(&NOUNSLIST, &(password_str[10..20])),
        choose_word_from_list(&VERBSLIST, &(password_str[20..30])),
        choose_word_from_list(&NOUNSLIST, &(password_str[30..40]))
    )
}

/// Generate and revoke a password
fn revoke(domain: String) {
    let pw_scrypt = get_scrypt_z85(domain);
    let pw_revocation_hash = get_revocation_hash(&pw_scrypt);
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
    eprintln!("xkcdget.legacy {XKCDGET_VERSION}");
    let mut args = args();
    match args.nth(1) {
        // no argument = interactive mode
        None => println!("{}", xkcdget(get_domain())),

        // the first argument is either an action flag or a domain
        Some(arg) => match arg.as_str() {
            // known action flags
            "-r" | "--revoke" => revoke(args.next().unwrap_or_else(get_domain)),
            // not a known action flag, so treat as a domain
            _ => println!("{}", xkcdget(arg)),
        },
    }
}
