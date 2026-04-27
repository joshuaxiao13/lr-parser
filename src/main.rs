mod parsing;

use parsing::{Cfg, Lr1Parser, Production};
use std::collections::HashSet;
use std::io;

/*
The goal is to write LR(0), SLR(1), LALR(1), LR(1) parsers (maybe LL(1) too).
Given a context free grammar, determine if the grammar is LR(1) and parse token sequence.
Common functionality should be consolidated to avoid duplicate code.
*/

/*
.cfg file should look like

# non-terminals

# terminals

# number of production rules

# Start non-terminal
*/

fn read_n_symbols(num_lines: usize) -> HashSet<String> {
    let mut symbols = HashSet::new();
    for _ in 0..num_lines {
        let mut s = String::new();
        io::stdin()
            .read_line(&mut s)
            .expect("Expected to read symbol");
        s = s.trim().to_string();
        if s.is_empty() {
            eprintln!("Symbol is empty");
            std::process::exit(1);
        }
        if s.contains(" ") {
            eprintln!("Symbol contains space");
            std::process::exit(1);
        }
        if symbols.contains(&s) {
            eprintln!("Duplicate symbol: {s}");
            std::process::exit(1);
        }
        symbols.insert(s);
    }

    symbols
}

fn read_unsigned_int() -> io::Result<usize> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    line.trim().parse::<usize>().map_err(|e| {
        eprintln!("Expected to read integer: {e}");
        std::process::exit(1);
    })
}

#[allow(non_snake_case)]
fn main() -> io::Result<()> {
    let N = read_unsigned_int()?;
    let nonterminals = read_n_symbols(N);

    let T = read_unsigned_int()?;

    let terminals = read_n_symbols(T);

    let R = read_unsigned_int()?;

    let production_rules = (0..R)
        .map(|_| {
            let buf = &mut String::new();
            io::stdin().read_line(buf)?;
            Ok(Production::parse(buf, &nonterminals))
        })
        .collect::<io::Result<Vec<Production>>>()?;

    let mut start_symbol = String::new();
    io::stdin().read_line(&mut start_symbol)?;
    start_symbol = start_symbol.trim().to_string();

    if !nonterminals.is_disjoint(&terminals) {
        let duplicate_symbols: HashSet<&String> = nonterminals.intersection(&terminals).collect();
        eprintln!("Duplicate terminal and non-terminal: {duplicate_symbols:?}");
        std::process::exit(1);
    }

    // Verify terminal and non-terminal symbols don't overlap
    // Report unused symbols in production rules
    // Report rules that can never be applied?

    // Verify all symbols in production rules are defined as either terminal or non-terminal
    // Verify lhs of production rules are non-terminals

    let cfg = Cfg::new(start_symbol, terminals, nonterminals, production_rules);
    let _parser = Lr1Parser::new(&cfg);

    Ok(())
}
