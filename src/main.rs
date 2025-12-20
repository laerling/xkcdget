use rpassword::prompt_password;
use scrypt::{scrypt, Params};
use std::cmp::max;
use std::env::{args, var};
use std::fs::{read_to_string, OpenOptions};
use std::io::{stdin, stdout, BufRead, ErrorKind, IsTerminal, Write};

mod wordlist;
use wordlist::WORDLIST;

// program version - adapt after every change!
const XKCDGET_VERSION: &str = "3.0.0-alpha.1"; // semantic versioning!

// utility constants
const USIZE_BYTES: usize = usize::BITS as usize / 8;
const DEBUG: u8 = 0;

// parameters
const WORDLIST_LEN: usize = 2048;
const AMOUNT_WORDS: u8 = 5;
const DEFAULT_PIN_LEN: u8 = 4;
const REVOCATION_LIST_FILENAME: &str = ".xkcdget-revocation";

/// calculate amount of bytes needed to choose one word from the wordlist
fn needed_bytes_per_word() -> usize {
    let needed_bits = usize::BITS - (WORDLIST_LEN-1).leading_zeros();
    (needed_bits as usize + 7) / 8
}

/// Return path to revocation file
fn get_revocation_filename() -> String {
    let homedir = var("HOME").expect("HOME environment variable unset or invalid");
    format!("{}/{}", homedir, REVOCATION_LIST_FILENAME)
}

/// Calculate the hash used for revocation.
fn get_revocation_hash(key: &[u8]) -> String {
    sha256::digest(key)
}

/// calculate and print password entropy
fn print_entropy() {
    // we can't use the bitshift method because we actually want to know fractional bits here, not
    // the amount of bits needed to choose a word.
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
fn get_key(domain: &String, master_pw: &String) -> Vec<u8> {
    // define scrypt parameters
    let log_n = 17; // FIXME experiment - it should take >1s on my beefy PC, but <20s on phone
                    // TODO Also check parameters for memory usage.
    let (r, p) = (8, 16);

    // calculate amount of bytes needed for key
    let key_len = max(needed_bytes_per_word() * AMOUNT_WORDS as usize, 10);

    // check scrypt parameters
    assert!(log_n >= 17, "It is recommended to set log_n to at least 17");
    assert!(key_len >= 10, "Key length must be 10 or more bytes");
    assert!(key_len <= 64, "Key length must be 64 or less bytes");
    let scrypt_params = Params::new(log_n, r, p, key_len).expect("Cannot create scrypt parameters");

    // vector to contain key
    let mut key = Vec::new();
    key.resize(key_len, 0);

    // generate key until one is found that has not been revoked
    let revocation_hashes = get_revocation_hashes();
    for iteration in 0.. {
        // get password for this iteration
        let salt = format!("{}:{}", domain, iteration);
        scrypt(
            master_pw.as_bytes(),
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
    // FIXME replace wordlist with a longer one
    assert!(WORDLIST.len() == WORDLIST_LEN);

    // Problem:
    // Mapping [1,2^b] (where b is an arbitrary amount of bits from the scrypt-derived key) to [1,N]
    // (where N is the amount of entries in the wordlist and N<=2^b) can only result in a uniform
    // probability distribution if N is a power of two. Otherwise we need to
    // 1) either map the remaining 2^b mod N preimage values to 2^b mod N < N wordlist entries,
    //    which results in some entries having a higher probability of being chosen than others,
    // 2) or we need to "reroll" the value in [1,2^b] until it lies within [1,N].
    //
    // Option 1 always produces a non-uniform probability distribution over [1,N], but the
    // difference between the probability of the 2^b mod N values and the probability of the others
    // can be minimized by choosing a higher value for b.
    // => 2^b mod N values will have a probability of ceil(2^b/N)/(2^b)
    // => the other N - 2^b mod N values will have a probability floor(2^b/N)/(2^b)
    // E. g. for b=12 and N=2047 the probability difference is ~0.024%pt, for b=16 just ~0.0015%pt
    //
    // Option 2 can produce a uniform probability distribution over [1,N], but only when the PRNG
    // used for "rerolling" the bits does not only depend of the bits themselves. Otherwise (that
    // is, if the result of the reroll depends only on the current bits) all values in [2^b-(2^b
    // mod N)+1, 2^b] will always be mapped to the same values in [1,N], which is just option 1
    // again, but with a different distribution.
    // Therefore to realize option 2, the PRNG must depend on both the bits of the current roll as
    // well as other values - namely the domain, master password, iteration, word index, and the
    // scrypt-derived key. Also, the PRNG must have the property that the change of just one bit in
    // these input data sources must result in a 50% flip probability for every output bit. These
    // properties are fulfilled by hash functions.
    //
    // One might think that this is just option 1 but with a different distribution of mappings
    // from [2^b-(2^b mod N)+1, 2^b] to [1,N]. That is correct for one single roll. But since the
    // distribution itself is different for every roll, it averages out - since every wordlist
    // entry can either be part of the distribution (in 2^b mod N cases) or not (in all other
    // cases), these two different probabilities, weighted by the amount of corresponding cases,
    // average to 1/N.

    // make sure the amount of bytes needed to choose a word fit into a usize value
    let bytes_per_word = needed_bytes_per_word();
    assert!(bytes_per_word <= USIZE_BYTES);

    // get scrypt-derived key
    let master_pw = get_master_password();
    let key = get_key(&domain, &master_pw);
    if DEBUG >= 1 {
        println!("scrypt-derived key == {:?}", key);
    }

    // choose words
    let mut words = Vec::new();
    for word_i in 0..AMOUNT_WORDS {

        // copy needed amount of bytes from scrypt-derived key to transient word key
        let offset = word_i as usize * bytes_per_word;
        let mut word_key: [u8; USIZE_BYTES] = [0; USIZE_BYTES];
        word_key[USIZE_BYTES-bytes_per_word..].copy_from_slice(
            &key[offset..(offset + bytes_per_word)]);
        if DEBUG >= 1 {
            println!("word_key == {:?}", word_key);
        }

        // recalculate index until it's within the desired range
        let mut index = usize::from_be_bytes(word_key);
        let mut iteration: u64 = 0; // hash iterations
        let mut tries: u64 = 0; // index decoding tries
        let mut hash: Vec<u8> = Vec::new();
        loop {
            if DEBUG >= 2 {
                println!("  index == {}", index);
            }

            // it's in the needed range
            if index < WORDLIST_LEN {
                break;
            }

            // Try for another.
            // even after just a thousand iterations it's astronomically improbable for the result
            // to not have landed in the needed range at least once
            iteration += 1;
            assert!(iteration < 1000);
            if DEBUG >= 1 {
                println!(" Iteration {}", iteration);
            }

            // it's not in the needed range - reroll, that is, rehash with all variable inputs
            let mut hash_inputs: Vec<u8> = Vec::with_capacity(256);
            // last hash
            hash_inputs.extend_from_slice(&hash);
            // current word key
            hash_inputs.extend_from_slice(&word_key);
            // complete scrypt-derived key
            hash_inputs.extend_from_slice(key.as_slice());
            // domain
            hash_inputs.extend_from_slice(domain.as_bytes());
            // master password
            hash_inputs.extend_from_slice(master_pw.as_bytes());
            // current iteration
            hash_inputs.extend_from_slice(iteration.to_string().as_bytes());
            // word position
            hash_inputs.push(word_i);

            // hash all variable inputs from above
            if DEBUG >= 2 {
                println!("  hash_inputs == {:?}", hash_inputs);
            }
            let hash_hex = sha256::digest(hash_inputs);
            if DEBUG >= 2 {
                println!("  hash == {}", hash_hex);
            }
            hash = hex::decode(hash_hex).expect(
                "Cannot decode hexadecimal representation of sha256 hash");
            if DEBUG >= 2 {
                println!("  hash == {:?}", hash);
            }

            // iterate over bytes of hash
            let n_chunks = 256 / 8 / bytes_per_word;
            for chunk_i in 0..n_chunks {
                tries += 1;
                let offset = chunk_i * bytes_per_word;
                let word_key_slice = &hash[offset..(offset + bytes_per_word)];
                if DEBUG >= 2 {
                    println!("  word_key_slice == {:?}", word_key_slice);
                }

                // copy needed amount of bytes from hash to word_key
                word_key = [0; USIZE_BYTES];
                word_key[USIZE_BYTES-bytes_per_word..].copy_from_slice(word_key_slice);

                // calculate index from word_key and check
                index = usize::from_be_bytes(word_key);
                if DEBUG >= 2 {
                    println!("  index == {}", index);
                }
                if index < WORDLIST_LEN {
                    break;
                }
            }
        }

        // choose word
        assert!(index < WORDLIST_LEN);
        if DEBUG >= 1 {
            println!("Index fits after {tries} tries ({iteration} iterations)");
        }
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
fn pin(domain: String, _digits: u8) {
    let master_pw = get_master_password();
    let _key = get_key(&domain, &master_pw);
    todo!("choose digits");
}

/// Generate and revoke a password
fn revoke(domain: String) {
    let master_pw = get_master_password();
    let key = get_key(&domain, &master_pw);
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
    if DEBUG >= 1 {
        print_entropy();
    }
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
