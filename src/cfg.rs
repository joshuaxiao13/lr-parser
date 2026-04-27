use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Symbol {
    Terminal(String),
    NonTerminal(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Production {
    pub nonterminal: String,
    pub expansion: Vec<Symbol>,
}

impl Production {
    pub fn new(nonterminal: String, expansion: Vec<Symbol>) -> Self {
        Self {
            nonterminal,
            expansion,
        }
    }

    pub fn parse(line: &str, nonterminals: &HashSet<String>) -> Self {
        let line = line.trim();
        let parts = line.split_ascii_whitespace().collect::<Vec<&str>>();
        match parts.as_slice() {
            [nonterminal, "->", rest @ ..] => {
                let expansion = rest
                    .iter()
                    .map(|x| {
                        let s = x.to_string();
                        if nonterminals.contains(&s) {
                            Symbol::NonTerminal(s)
                        } else {
                            Symbol::Terminal(s)
                        }
                    })
                    .collect();
                Self {
                    nonterminal: nonterminal.to_string(),
                    expansion,
                }
            }
            _ => {
                panic!(
                    "Cannot parse production: {}. Expected [non-terminal] -> ...",
                    line
                );
            }
        }
    }
}

#[derive(Debug)]
pub struct Cfg {
    start_symbol: String,
    nonterminals: HashSet<String>,
    terminals: HashSet<String>,
    productions: Vec<Production>,
}

impl Cfg {
    pub fn new(
        start_symbol: String,
        nonterminals: HashSet<String>,
        terminals: HashSet<String>,
        productions: Vec<Production>,
    ) -> Self {
        Self {
            start_symbol,
            nonterminals,
            terminals,
            productions,
        }
    }

    fn augment_grammar(&self) -> AugmentedCfg {
        let append_until_unique = |mut s: String| -> String {
            while self.nonterminals.contains(&s) || self.terminals.contains(&s) {
                // Append ticks until unique
                s += "'";
            }
            s
        };

        let new_start_symbol = append_until_unique(String::from("S'"));
        let end_terminal = append_until_unique(String::from("$"));

        let mut new_nonterminals = self.nonterminals.clone();
        new_nonterminals.insert(new_start_symbol.clone());

        let mut new_productions = self.productions.clone();
        new_productions.push(Production::new(
            new_start_symbol.clone(),
            vec![
                Symbol::NonTerminal(self.start_symbol.clone()),
                Symbol::Terminal(end_terminal.clone()),
            ],
        ));

        AugmentedCfg {
            start_symbol: new_start_symbol,
            end_terminal,
            nonterminals: new_nonterminals,
            terminals: self.terminals.clone(),
            productions: new_productions,
        }
    }
}

pub struct AugmentedCfg {
    pub start_symbol: String,
    pub end_terminal: String,
    pub nonterminals: HashSet<String>,
    pub terminals: HashSet<String>,
    pub productions: Vec<Production>,
}

pub struct CfgContext {
    pub augmented_cfg: AugmentedCfg,
    pub productions_by_nonterminal: HashMap<String, Vec<usize>>,
    pub nullable: HashSet<String>,
    pub first: HashMap<String, HashSet<String>>,
    follow: HashMap<String, HashSet<String>>,
}

pub fn is_nullable(symbols: &[Symbol], nullable: &HashSet<String>) -> bool {
    symbols.iter().all(|s| {
        if let Symbol::NonTerminal(nt) = s {
            nullable.contains(nt)
        } else {
            false
        }
    })
}

pub fn get_first(
    symbols: &[Symbol],
    first: &HashMap<String, HashSet<String>>,
    nullable: &HashSet<String>,
) -> Vec<String> {
    let mut first_set = Vec::new();
    for i in 0..symbols.len() {
        if !is_nullable(&symbols[0..i], nullable) {
            continue;
        }
        match &symbols[i] {
            Symbol::Terminal(t) => {
                first_set.push(t.clone());
                break;
            }
            Symbol::NonTerminal(nt) => {
                if let Some(nt_first) = first.get(nt) {
                    first_set.extend(nt_first.iter().cloned());
                }
            }
        }
    }
    first_set
}

impl From<&Cfg> for CfgContext {
    fn from(cfg: &Cfg) -> Self {
        let cfg = cfg.augment_grammar();
        let mut productions_by_nonterminal: HashMap<_, Vec<usize>> = HashMap::new();
        // first(A) = {a in T : A =>* a... }
        let mut first = HashMap::new();
        // nullable(A) = A =>* epsilon
        let mut nullable = HashSet::new();
        // follow(A) = {a in T : S =>* ...Aa... }
        let mut follow: HashMap<String, HashSet<String>> = HashMap::new();

        for (idx, p) in cfg.productions.iter().enumerate() {
            productions_by_nonterminal
                .entry(p.nonterminal.to_owned())
                .or_default()
                .push(idx);
        }

        loop {
            // Tracks whether any of first, nullable, follow sets changed
            let mut changed = false;

            for p in cfg.productions.iter() {
                // Nullable
                if is_nullable(&p.expansion, &nullable) {
                    changed |= nullable.insert(p.nonterminal.clone());
                }

                // First
                for t in get_first(&p.expansion, &first, &nullable) {
                    changed |= first
                        .entry(p.nonterminal.clone())
                        .or_default()
                        .insert(t.clone());
                }

                // Follow
                for i in 0..p.expansion.len() {
                    if let Symbol::NonTerminal(cur_symbol) = &p.expansion[i] {
                        let tail = &p.expansion[i + 1..];
                        for t in get_first(tail, &first, &nullable) {
                            changed |= follow
                                .entry(cur_symbol.clone())
                                .or_default()
                                .insert(t.clone());
                        }
                        if is_nullable(tail, &nullable)
                            && let Some(follow_set) = follow.get(&p.nonterminal).cloned()
                        {
                            for t in follow_set {
                                changed |= follow
                                    .entry(cur_symbol.clone())
                                    .or_default()
                                    .insert(t.clone());
                            }
                        }
                    }
                }

                // Update last symbol's follow seperately
                if let Some(Symbol::NonTerminal(last_nt)) = p.expansion.last() {
                    // For A -> ...B, follow(A) is a subset of follow(B)
                    for t in follow.get(&p.nonterminal).cloned().into_iter().flatten() {
                        changed |= follow.entry(last_nt.clone()).or_default().insert(t);
                    }
                }
            }

            if !changed {
                break;
            }
        }

        Self {
            augmented_cfg: cfg,
            productions_by_nonterminal,
            first,
            nullable,
            follow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cfg_context() {
        let start_symbol = "S".to_string();
        let nonterminals = vec!["S", "E", "A", "B", "C"]
            .into_iter()
            .map(String::from)
            .collect();
        let terminals = vec!["(", ")", "a", "b", "c"]
            .into_iter()
            .map(String::from)
            .collect();
        let productions = vec![
            Production::new(
                "S".to_string(),
                vec![
                    Symbol::Terminal("(".to_string()),
                    Symbol::NonTerminal("E".to_string()),
                    Symbol::Terminal(")".to_string()),
                ],
            ),
            Production::new(
                "E".to_string(),
                vec![
                    Symbol::NonTerminal("A".to_string()),
                    Symbol::NonTerminal("B".to_string()),
                    Symbol::NonTerminal("C".to_string()),
                ],
            ),
            Production::new("A".to_string(), vec![Symbol::Terminal("a".to_string())]),
            Production::new("A".to_string(), vec![]),
            Production::new("B".to_string(), vec![Symbol::Terminal("b".to_string())]),
            Production::new("B".to_string(), vec![]),
            Production::new("C".to_string(), vec![Symbol::Terminal("c".to_string())]),
            Production::new("C".to_string(), vec![]),
        ];
        let cfg = Cfg::new(start_symbol, nonterminals, terminals, productions);
        let ctx = CfgContext::from(&cfg);

        assert_eq!(ctx.augmented_cfg.start_symbol, "S'");

        // Nullable
        assert_eq!(
            ctx.nullable,
            HashSet::from([
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "E".to_string()
            ])
        );
        // First
        assert_eq!(
            ctx.first.get(&"S'".to_string()),
            Some(&HashSet::from(["(".to_string()]))
        );
        assert_eq!(
            ctx.first.get(&"S".to_string()),
            Some(&HashSet::from(["(".to_string()]))
        );
        assert_eq!(
            ctx.first.get(&"E".to_string()),
            Some(&HashSet::from([
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ]))
        );
        assert_eq!(
            ctx.first.get(&"A".to_string()),
            Some(&HashSet::from(["a".to_string()]))
        );
        assert_eq!(
            ctx.first.get(&"B".to_string()),
            Some(&HashSet::from(["b".to_string()]))
        );
        assert_eq!(
            ctx.first.get(&"C".to_string()),
            Some(&HashSet::from(["c".to_string()]))
        );
        // Follow
        assert_eq!(ctx.follow.get(&"S'".to_string()), None,);
        assert_eq!(
            ctx.follow.get(&"S".to_string()),
            Some(&HashSet::from(["$".to_string()])),
        );

        assert_eq!(
            ctx.follow.get(&"E".to_string()),
            Some(&HashSet::from([")".to_string()]))
        );
        assert_eq!(
            ctx.follow.get(&"A".to_string()),
            Some(&HashSet::from([
                "b".to_string(),
                "c".to_string(),
                ")".to_string()
            ]))
        );
        assert_eq!(
            ctx.follow.get(&"B".to_string()),
            Some(&HashSet::from(["c".to_string(), ")".to_string()]))
        );
        assert_eq!(
            ctx.follow.get(&"C".to_string()),
            Some(&HashSet::from([")".to_string()]))
        );
    }
}
