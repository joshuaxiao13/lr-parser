use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

// Given CFG <N, T, R, S>, build LR(0) DFA

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Symbol {
    Terminal(String),
    NonTerminal(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Production {
    nonterminal: String,
    expansion: Vec<Symbol>,
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
                eprintln!("Cannot parse production: {line:?}. Expected [non-terminal] -> ...");
                std::process::exit(1);
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
}

struct AugmentedCfg {
    start_symbol: String,
    end_terminal: String,
    nonterminals: HashSet<String>,
    terminals: HashSet<String>,
    productions: Vec<Production>,
}

struct CfgContext {
    augmented_cfg: AugmentedCfg,
    productions_by_nonterminal: HashMap<String, Vec<usize>>,
    nullable: HashSet<String>,
    first: HashMap<String, HashSet<String>>,
    follow: HashMap<String, HashSet<String>>,
}

fn is_nullable(symbols: &[Symbol], nullable: &HashSet<String>) -> bool {
    symbols.iter().all(|s| {
        if let Symbol::NonTerminal(nt) = s {
            nullable.contains(nt)
        } else {
            false
        }
    })
}

fn get_first(
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

impl CfgContext {
    fn augment_grammar(cfg: &Cfg) -> AugmentedCfg {
        let append_until_unique = |mut s: String| -> String {
            while cfg.nonterminals.contains(&s) || cfg.terminals.contains(&s) {
                // Append underscores until unique
                s += "'";
            }
            s
        };

        let new_start_symbol = append_until_unique(String::from("S'"));
        let end_terminal = append_until_unique(String::from("$"));

        let mut new_nonterminals = cfg.nonterminals.clone();
        new_nonterminals.insert(new_start_symbol.clone());

        let mut new_productions = cfg.productions.clone();
        new_productions.push(Production::new(
            new_start_symbol.clone(),
            vec![
                Symbol::NonTerminal(cfg.start_symbol.clone()),
                Symbol::Terminal(end_terminal.clone()),
            ],
        ));

        AugmentedCfg {
            start_symbol: new_start_symbol,
            end_terminal,
            nonterminals: new_nonterminals,
            terminals: cfg.terminals.clone(),
            productions: new_productions,
        }
    }

    fn from(cfg: &Cfg) -> Self {
        let cfg = CfgContext::augment_grammar(cfg);
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

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct LrItem {
    production_id: usize,
    /// Position of dot (the number of symbols seen in the production)
    dot: usize,
    lookahead: BTreeSet<String>,
}

impl LrItem {
    fn get_symbol_after_dot<'a>(&self, cfgctx: &'a CfgContext) -> Option<&'a Symbol> {
        let production = &cfgctx.augmented_cfg.productions[self.production_id];
        production.expansion.get(self.dot)
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct DfaState {
    items: BTreeSet<LrItem>,
}

struct Dfa {
    states: Vec<DfaState>,
    transitions: HashMap<(usize, Symbol), usize>,
}

pub struct Lr1Parser {
    cfgctx: CfgContext,
    dfa: Dfa,
}

impl Lr1Parser {
    pub fn new(cfg: &Cfg) -> Self {
        let cfgctx = CfgContext::from(&cfg);
        let dfa = Lr1Parser::build_dfa(&cfgctx);
        let parser = Self { cfgctx, dfa };
        parser.print_dfa();
        parser.check_conflicts();
        parser
    }

    fn build_dfa(cfgctx: &CfgContext) -> Dfa {
        let start_transition = &cfgctx
            .productions_by_nonterminal
            .get(&cfgctx.augmented_cfg.start_symbol);

        let Some([start_production_idx]) = start_transition.map(|v| v.as_slice()) else {
            panic!(
                "Expected exactly one production for the augmented start symbol {}",
                cfgctx.augmented_cfg.start_symbol
            );
        };

        let nfa_start_state = LrItem {
            production_id: *start_production_idx,
            dot: 0,
            lookahead: BTreeSet::from([cfgctx.augmented_cfg.end_terminal.clone()]),
        };

        let dfa_start_state = DfaState {
            items: Lr1Parser::epsilon_closure(nfa_start_state, cfgctx),
        };

        let mut states = vec![dfa_start_state];
        let mut queue = VecDeque::from([0]);
        let mut transitions = HashMap::new();

        while let Some(state_id) = queue.pop_front() {
            // Transition
            let mut closure_by_tansition: HashMap<_, BTreeSet<_>> = HashMap::new();

            for item in &states[state_id].items {
                if let Some(s) = item.get_symbol_after_dot(cfgctx) {
                    let shift_item = LrItem {
                        production_id: item.production_id,
                        dot: item.dot + 1,
                        lookahead: item.lookahead.clone(),
                    };
                    let eps_closure = Lr1Parser::epsilon_closure(shift_item, cfgctx);
                    closure_by_tansition
                        .entry(s)
                        .or_default()
                        .extend(eps_closure);
                }
            }

            for (symbol, closure) in closure_by_tansition {
                let next_state = DfaState { items: closure };

                if states.contains(&next_state) {
                    continue;
                }

                states.push(next_state);
                let next_state_id = states.len() - 1;
                queue.push_back(next_state_id);
                transitions.insert((state_id, symbol.clone()), next_state_id);
            }
        }

        // println!("# states = {:?}", states.len());
        // println!("states = {:#?}", states);
        // println!();

        // println!("# transitions = {:?}", transitions.len());
        // println!("transitions = {:#?}", transitions);
        // println!();

        Dfa {
            states,
            transitions,
        }
    }

    /// Get the NFA epsilon closure for a LR item
    fn epsilon_closure(item: LrItem, cfgctx: &CfgContext) -> BTreeSet<LrItem> {
        let mut lookahead_by_production_id_and_dot =
            HashMap::from([((item.production_id, item.dot), item.lookahead)]);
        let mut deque = VecDeque::from([(item.production_id, item.dot)]);

        while let Some((production_id, dot)) = deque.pop_front() {
            let item = LrItem {
                production_id,
                dot,
                lookahead: lookahead_by_production_id_and_dot
                    .get(&(production_id, dot))
                    .unwrap_or(&BTreeSet::new())
                    .clone(),
            };
            match item.get_symbol_after_dot(cfgctx) {
                Some(Symbol::NonTerminal(nt)) => {
                    let production = &cfgctx.augmented_cfg.productions[item.production_id];
                    let tail = &production.expansion[item.dot + 1..];
                    let mut new_lookahead_candidates: BTreeSet<_> =
                        get_first(tail, &cfgctx.first, &cfgctx.nullable)
                            .into_iter()
                            .collect();

                    if is_nullable(tail, &cfgctx.nullable) {
                        new_lookahead_candidates.extend(item.lookahead.clone());
                    }

                    for reduce_idx in cfgctx
                        .productions_by_nonterminal
                        .get(nt)
                        .into_iter()
                        .flatten()
                    {
                        let reduce_key = &(*reduce_idx, 0);
                        let new_lookaheads = if let Some(existing_lookaheads) =
                            lookahead_by_production_id_and_dot.get(reduce_key)
                        {
                            new_lookahead_candidates
                                .difference(existing_lookaheads)
                                .cloned()
                                .collect()
                        } else {
                            new_lookahead_candidates.clone()
                        };

                        if new_lookaheads.is_empty() {
                            continue;
                        }

                        lookahead_by_production_id_and_dot
                            .entry((*reduce_idx, 0))
                            .or_default()
                            .extend(new_lookaheads);

                        deque.push_back((*reduce_idx, 0));
                    }
                }
                Some(Symbol::Terminal(_)) | None => ( /* noop */),
            }
        }

        lookahead_by_production_id_and_dot
            .into_iter()
            .map(|((production_id, dot), lookahead)| LrItem {
                production_id,
                dot,
                lookahead,
            })
            .collect()
    }

    fn format_lr_item(&self, item: &LrItem) -> String {
        let production = &self.cfgctx.augmented_cfg.productions[item.production_id];
        let mut fmt_str = format!("{} ->", production.nonterminal);

        let mut expansion_with_dot = production.expansion.clone();
        expansion_with_dot.insert(item.dot, Symbol::Terminal(String::from("•")));
        expansion_with_dot.iter().for_each(|symbol| match symbol {
            Symbol::NonTerminal(s) | Symbol::Terminal(s) => fmt_str += format!(" {}", s).as_str(),
        });
        let fmt_lookahead = item
            .lookahead
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        fmt_str += format!(" : [ {} ]", fmt_lookahead).as_str();
        fmt_str
    }

    fn print_dfa(&self) -> () {
        for (state_id, state) in self.dfa.states.iter().enumerate() {
            println!("State {state_id}");
            for item in &state.items {
                println!("{}", self.format_lr_item(item));
            }
            println!()
        }
    }

    fn report_reduce_reduce_conflict(&self, state_id: usize, lookahead: &str) -> ! {
        let state = &self.dfa.states[state_id];
        let conflicting_reduces = state.items.iter().filter(|item| {
            item.get_symbol_after_dot(&self.cfgctx) == None && item.lookahead.contains(lookahead)
        });
        eprintln!(
            "Reduce-reduce conflict in state {}: lookahead {}",
            state_id, lookahead
        );
        conflicting_reduces.for_each(|item| eprintln!("{}", self.format_lr_item(item)));
        std::process::exit(1);
    }

    fn report_shift_reduce_conflict(&self, state_id: usize, lookahead: &str) -> ! {
        let state = &self.dfa.states[state_id];
        let conflicting_items =
            state
                .items
                .iter()
                .filter(|item| match item.get_symbol_after_dot(&self.cfgctx) {
                    None => item.lookahead.contains(lookahead),
                    Some(Symbol::Terminal(t)) => t == lookahead,
                    _ => false,
                });
        eprintln!(
            "Shift-reduce conflict in state {}: lookahead {}",
            state_id, lookahead
        );
        conflicting_items.for_each(|item| eprintln!("{}", self.format_lr_item(item)));
        std::process::exit(1);
    }

    fn check_conflicts(&self) -> () {
        for (state_id, state) in self.dfa.states.iter().enumerate() {
            let items = &state.items;
            // Find reduce-reduce conflict
            // Iterate through all items that are reduces, track lookahead sets
            let mut reduce_lookaheads = HashSet::new();
            items.iter().for_each(|item| {
                if let None = item.get_symbol_after_dot(&self.cfgctx) {
                    for lookahead in &item.lookahead {
                        if !reduce_lookaheads.insert(lookahead) {
                            self.report_reduce_reduce_conflict(state_id, lookahead);
                        }
                    }
                }
            });

            // Find shift-reduce conflict
            // Iterate through all items, track lookaheads and terminal transitions
            items.iter().for_each(|item| {
                if let Some(Symbol::Terminal(t)) = item.get_symbol_after_dot(&self.cfgctx) {
                    if reduce_lookaheads.contains(t) {
                        self.report_shift_reduce_conflict(state_id, t);
                    }
                }
            });
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
