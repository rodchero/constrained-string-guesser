use itertools::Itertools;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // get wordlist
    let mut wordlist: Vec<String> = vec![];
    let path = Path::new("/usr/share/dict/words");
    if let Ok(lines) = read_lines(path) {
        for line in lines.map_while(Result::ok) {
            if meets_criteria(&line) {
                wordlist.push(line);
            }
        }
    }
    let xor_constraint: Vec<u8> = vec![
        0x13, 0x1d, 0x04, 0x52, 0x01, 0x4e, 0x11, 0x05, 0x1c, 0x00, 0x19, 0x1c, 0x53, 0x0a, 0x18,
        0x06, 0x1c, 0x0b,
    ];
    // create shared output file handle
    // Open (or create) the log file in append mode
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("output.txt")
        .unwrap();
    let output_file = Arc::new(Mutex::new(file));

    let mut handles = Vec::new();

    for thread_id in 0..8 {
        let output_file = Arc::clone(&output_file);
        let wordlist = wordlist.clone();
        let xor_constraint = xor_constraint.clone();
        let p1_space_indices = vec![3, 12];
        let p2_space_indices = vec![5];
        let space_alternate_letters = vec!['r', 'n', 's'];
        let handle = thread::spawn(move || {
            search_subtask(
                wordlist.clone(),
                p1_space_indices,
                p2_space_indices,
                space_alternate_letters,
                xor_constraint,
                thread_id,
                output_file,
            );
        });
        handles.push(handle);
        // temporary
        break;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // spawn threads to begin multithreaded search
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn meets_criteria(word: &str) -> bool {
    if word.len() > 14 {
        return false;
    }
    word.chars()
        .all(|c| c.is_ascii_alphabetic() && c.is_ascii_lowercase())
}

fn search_subtask(
    wordlist: Vec<String>,
    p1_space_indices: Vec<usize>,
    p2_space_indices: Vec<usize>,
    space_alternate_letters: Vec<char>,
    xor_constraint: Vec<u8>,
    thread_id: usize,
    output_file: Arc<Mutex<File>>,
) {
    // compute candidate word lengths from spacelist
    // sort wordlist into sublists of candidate lengths
    // backtracking search through all possibilities
    // 1. come up with p1 (based on space constraints and alternate letters)
    // 2. XOR with X to get candidate p2
    // 3. validate p2 (valid english words, letter constraints, alternate space constraints)

    // get all lengths of possible words in both phrases

    let mut p1_word_lengths: Vec<usize> = vec![];
    for i in 0..p1_space_indices.len() {
        if i == 0 {
            p1_word_lengths.push(p1_space_indices[i]);
        } else {
            p1_word_lengths.push(p1_space_indices[i] - p1_space_indices[i - 1] - 1);
        }
    }
    p1_word_lengths.push(17 - p1_space_indices.last().unwrap());

    let mut p2_word_lengths: Vec<usize> = vec![];
    for i in 0..p2_space_indices.len() {
        if i == 0 {
            p2_word_lengths.push(p2_space_indices[i]);
        } else {
            p2_word_lengths.push(p2_space_indices[i] - p2_space_indices[i - 1] - 1);
        }
    }
    p2_word_lengths.push(17 - p2_space_indices.last().unwrap());

    // sort wordlist into sublists of candidate lengths
    let mut p1_wordlists: Vec<Vec<&String>> = vec![];
    for wordlen in p1_word_lengths {
        p1_wordlists.push(wordlist.iter().filter(|w| w.len() == wordlen).collect());
    }

    let mut p2_wordlists: Vec<Vec<&String>> = vec![];
    for wordlen in p2_word_lengths {
        p2_wordlists.push(wordlist.iter().filter(|w| w.len() == wordlen).collect());
    }

    // calculate letter position constraints
    let mut p1_letter_index_cons: Vec<(char, usize)> = vec![];
    let mut p2_letter_index_cons: Vec<(char, usize)> = vec![];
    let mut all_space_indices: Vec<usize> = vec![];
    all_space_indices.extend(p1_space_indices.clone());
    all_space_indices.extend(p2_space_indices.clone());
    all_space_indices.sort();
    for i in 0..all_space_indices.len() {
        if p1_space_indices.contains(&all_space_indices[i]) {
            p1_letter_index_cons.push((space_alternate_letters[i], all_space_indices[i]));
        } else if p2_space_indices.contains(&all_space_indices[i]) {
            p2_letter_index_cons.push((space_alternate_letters[i], all_space_indices[i]));
        }
    }

    let mut searchspace: i64 = 1;
    for wl in &p1_wordlists {
        searchspace *= wl.len() as i64;
    }
    // Mutexd'd area

    println!("finished pre-calculations; starting search");
    println!("thread {thread_id} estimated search space: {searchspace}");

    let mut solutions_found = 0;

    // search through all the possible word combinations for p1
    for candidate_p1 in p1_wordlists
        .iter()
        .map(|s| s.iter())
        .multi_cartesian_product()
    {
        let p1: String = candidate_p1.iter().map(|s| s.as_str()).join(" ");

        // check constraints on p1 (space alternate letters), continue if false
        let mut p1_constraints_passed = true;
        for con in &p1_letter_index_cons {
            if p1.chars().nth(con.1).unwrap() != con.0 {
                p1_constraints_passed = false;
            }
        }
        if !p1_constraints_passed {
            continue;
        }

        // generate corresponding p2
        let p2: String = String::from_utf8(
            p1.as_bytes()
                .iter()
                .zip(xor_constraint.iter())
                .map(|(&x1, &x2)| x1 ^ x2)
                .collect(),
        )
        .unwrap();

        // check constraints on p2, if false, continue to next loop asap:

        // characters which appear opposite spaces in p1 are known
        let mut p2_constraints_passed = true;
        for con in &p2_letter_index_cons {
            if p2.chars().nth(con.1).unwrap() != con.0 {
                p2_constraints_passed = false;
            }
        }
        if !p2_constraints_passed {
            continue;
        }

        // shared char at index 9
        if p1.chars().nth(9).unwrap() != p2.chars().nth(9).unwrap() {
            continue;
        }

        // spaces at expected word boundaries (p2_space_indices)
        for space_idx in &p2_space_indices {
            if p2.chars().nth(*space_idx).unwrap() != ' ' {
                p2_constraints_passed = false;
            }
        }
        if !p2_constraints_passed {
            continue;
        }

        // each word in p2 is in wordlist
        let p2_words: Vec<&str> = p2.split_ascii_whitespace().collect();
        for i in 0..p2_words.len() {
            if !p2_wordlists[i].contains(&&String::from(p2_words[i])) {
                p2_constraints_passed = false;
            }
        }
        if !p2_constraints_passed {
            continue;
        }

        // write result to output file and print solution number

        solutions_found += 1;

        // mutex'd area
        {
            let mut file = output_file.lock().unwrap();
            _ = writeln!(
                *file,
                "Thread {thread_id} discovered solution: p1: {p1}, p2: {p2}"
            );
            println!(
                "Thread {thread_id} found solution number {solutions_found}, p1: {p1}, p2: {p2}"
            );
        }
    }
    //mutex'd area
    println!(
        "Thread {thread_id} finished search space. Total of {solutions_found} solutions found."
    );
}
