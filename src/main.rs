use itertools::Itertools;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Mutex;
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
    search_subtask(
        wordlist,
        vec![3, 12],
        vec![5],
        vec!['r', 'n', 's'],
        xor_constraint,
    );
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

    println!("finished pre-calculations; starting search");

    // search through all the possible word combinations for p1
    for candidate_p1 in p1_wordlists
        .iter()
        .map(|s| s.iter())
        .multi_cartesian_product()
    {
        let p1: String = candidate_p1.iter().map(|s| s.as_str()).join(" ");

        // check constraints on p1 (space alternate letters), continue if false
        todo!();

        // generate corresponding p2
        let p2: String = String::from_utf8(
            p1.as_bytes()
                .iter()
                .zip(xor_constraint.iter())
                .map(|(&x1, &x2)| x1 ^ x2)
                .collect(),
        )
        .unwrap();

        // check constraints on p2
        todo!();

        // continue to next iteration if constraints not met, else write to output file (mutex'd)
    }
}
