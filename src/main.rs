use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};

mod cfg;
mod parsing;

use cfg::{Cfg, Production};
use parsing::Lr1Parser;

/*
The goal is to write SLR(1), LALR(1), LR(1) parsers.
Given a context free grammar, determine if the grammar is LR(1) and parse token sequence.

.cfg file should look like

# non-terminals

# terminals

# number of production rules

# Start non-terminal
*/

fn read_n_symbols(reader: &mut BufReader<File>, num_lines: usize) -> HashSet<String> {
    let mut symbols = HashSet::new();
    for _ in 0..num_lines {
        let mut s = String::new();
        reader.read_line(&mut s).expect("Expected to read symbol");
        s = s.trim().to_string();
        if s.is_empty() {
            panic!("Symbol is empty");
        }
        if s.contains(" ") {
            panic!("Symbol contains space");
        }
        if symbols.contains(&s) {
            panic!("Duplicate symbol: {s}");
        }
        symbols.insert(s);
    }

    symbols
}

fn read_unsigned_int(reader: &mut BufReader<File>) -> io::Result<usize> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    line.trim()
        .parse::<usize>()
        .map_err(|e| panic!("Expected to read integer: {e}"))
}

#[allow(non_snake_case)]
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    let grammar_file = &args[1];
    let input_file = args.get(2);

    let file = File::open(grammar_file)?;
    let mut grammar_reader = BufReader::new(file);

    let N = read_unsigned_int(&mut grammar_reader)?;
    let nonterminals = read_n_symbols(&mut grammar_reader, N);

    let T = read_unsigned_int(&mut grammar_reader)?;

    let terminals = read_n_symbols(&mut grammar_reader, T);

    let R = read_unsigned_int(&mut grammar_reader)?;

    let production_rules = (0..R)
        .map(|_| {
            let buf = &mut String::new();
            grammar_reader.read_line(buf)?;
            Ok(Production::parse(buf, &nonterminals))
        })
        .collect::<io::Result<Vec<_>>>()?;

    let mut start_symbol = String::new();
    grammar_reader.read_line(&mut start_symbol)?;
    start_symbol = start_symbol.trim().to_string();

    if !nonterminals.is_disjoint(&terminals) {
        let duplicate_symbols: HashSet<_> = nonterminals.intersection(&terminals).collect();
        panic!("Duplicate terminal and non-terminal: {duplicate_symbols:?}");
    }

    // Verify terminal and non-terminal symbols don't overlap
    // Report unused symbols in production rules
    // Report rules that can never be applied?

    // Verify all symbols in production rules are defined as either terminal or non-terminal
    // Verify lhs of production rules are non-terminals

    let cfg = Cfg::new(start_symbol, terminals, nonterminals, production_rules);
    let parser = Lr1Parser::new(&cfg);

    if let Some(input_file) = input_file {
        let input = fs::read_to_string(input_file)?;
        let input: Vec<&str> = input.split_ascii_whitespace().collect(); // assume any whitespace separates tokens

        /* let tree = */
        parser.parse(&input);
        // println!("{:?}", tree);
    }

    Ok(())
}
