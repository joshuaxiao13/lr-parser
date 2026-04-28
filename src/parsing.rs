use crate::cfg::*;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, PartialEq)]
pub struct ParseTree {
    value: Symbol,
    children: Vec<ParseTree>,
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
        let cfgctx = CfgContext::from(cfg);
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

                let next_state_id = if let Some((id, _)) =
                    states.iter().enumerate().find(|(_, x)| *x == &next_state)
                {
                    id
                } else {
                    states.push(next_state);
                    let next_state_id = states.len() - 1;
                    queue.push_back(next_state_id);
                    next_state_id
                };

                transitions.insert((state_id, symbol.clone()), next_state_id);
            }
        }

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
                        let reduce_key = (*reduce_idx, 0);
                        let new_lookaheads = if let Some(existing_lookaheads) =
                            lookahead_by_production_id_and_dot.get(&reduce_key)
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
                            .entry(reduce_key)
                            .or_default()
                            .extend(new_lookaheads);

                        deque.push_back(reduce_key);
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

    fn reduce(&self, state_id: usize, lookahead: &str) -> Option<&Production> {
        let state = &self.dfa.states[state_id];
        state
            .items
            .iter()
            .find(|item| {
                item.get_symbol_after_dot(&self.cfgctx) == None
                    && item.lookahead.contains(lookahead)
            })
            .map(|item| &self.cfgctx.augmented_cfg.productions[item.production_id])
    }

    pub fn parse(&self, input: &[&str]) -> ParseTree {
        let mut stack: Vec<(ParseTree, usize)> = Vec::new();
        let mut right_derivation: Vec<&Production> = Vec::new();

        let get_next_state_id = |stack: &[(_, usize)], symbol: &Symbol| -> usize {
            let last_state_id = if let Some((_, state_id)) = stack.last() {
                *state_id
            } else {
                0
            };
            *self
                .dfa
                .transitions
                .get(&(last_state_id, symbol.clone()))
                .unwrap_or_else(|| {
                    dbg!(self.dfa.transitions.get(&(last_state_id, symbol.clone())));
                    panic!(
                        "Input is not in language of grammar: no transition on state {} and symbol {:?}",
                        last_state_id, symbol
                    )
                })
        };

        for t in input
            .iter()
            .chain([self.cfgctx.augmented_cfg.end_terminal.as_str()].iter())
        {
            let input_symbol = Symbol::Terminal(t.to_string());

            while let Some((_, state_id)) = stack.last()
                && let Some(reduce) = self.reduce(*state_id, t)
            {
                let reduce_symbol = Symbol::NonTerminal(reduce.nonterminal.clone());

                let stack_len = stack.len();
                let drain_start = stack_len - reduce.expansion.len();
                let new_state_id = get_next_state_id(&stack[..drain_start], &reduce_symbol);
                let symbols_to_reduce = stack.drain(drain_start..);

                let reduce_node = ParseTree {
                    value: reduce_symbol,
                    children: symbols_to_reduce.map(|(node, _)| node).collect(),
                };

                stack.push((reduce_node, new_state_id));
                right_derivation.push(reduce)
            }

            let next_state_id = get_next_state_id(&stack, &input_symbol);

            stack.push((
                ParseTree {
                    value: Symbol::Terminal(t.to_string()),
                    children: vec![],
                },
                next_state_id,
            ));
        }

        println!("Rightmost derivation:");
        for reduce in right_derivation.iter().rev() {
            println!("Apply {} -> {:?}", reduce.nonterminal, reduce.expansion);
        }

        stack
            .into_iter()
            .next()
            .map(|(tree, _)| tree)
            .expect("Stack should be non-empty after parsing is complete")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_test_cfg() -> Cfg {
        /*
         S -> Identifier
         S -> MethodCall
         Identifier -> id
         Identifier -> S . id
         MethodCall -> Identifier ( )
        */
        let start_symbol = "S".to_string();
        let nonterminals = HashSet::from([
            "S".to_string(),
            "Identifier".to_string(),
            "MethodCall".to_string(),
        ]);
        let terminals = HashSet::from([
            "id".to_string(),
            ".".to_string(),
            "(".to_string(),
            ")".to_string(),
        ]);
        let productions = vec![
            Production::new(
                "S".to_string(),
                vec![Symbol::NonTerminal("Identifier".to_string())],
            ),
            Production::new(
                "S".to_string(),
                vec![Symbol::NonTerminal("MethodCall".to_string())],
            ),
            Production::new(
                "Identifier".to_string(),
                vec![Symbol::Terminal("id".to_string())],
            ),
            Production::new(
                "Identifier".to_string(),
                vec![
                    Symbol::NonTerminal("S".to_string()),
                    Symbol::Terminal(".".to_string()),
                    Symbol::Terminal("id".to_string()),
                ],
            ),
            Production::new(
                "MethodCall".to_string(),
                vec![
                    Symbol::NonTerminal("Identifier".to_string()),
                    Symbol::Terminal("(".to_string()),
                    Symbol::Terminal(")".to_string()),
                ],
            ),
        ];
        Cfg::new(start_symbol, nonterminals, terminals, productions)
    }

    #[test]
    #[should_panic]
    fn input_not_in_language() {
        let cfg = get_test_cfg();
        let parser = Lr1Parser::new(&cfg);
        let input = ["id", "(", ")", "(", ")"];
        parser.parse(&input);
    }

    #[test]
    fn test_parse_simple_identifier() {
        let cfg = get_test_cfg();
        let parser = Lr1Parser::new(&cfg);
        let input = ["id"];
        let tree = parser.parse(&input);
        assert_eq!(
            tree,
            ParseTree {
                value: Symbol::NonTerminal("S".to_string()),
                children: vec![ParseTree {
                    value: Symbol::NonTerminal("Identifier".to_string()),
                    children: vec![ParseTree {
                        value: Symbol::Terminal("id".to_string()),
                        children: vec![]
                    }]
                }]
            }
        )
    }

    #[test]
    fn test_parse_qualified_identifier() {
        let cfg = get_test_cfg();
        let parser = Lr1Parser::new(&cfg);
        let input = ["id", ".", "id"];
        let tree = parser.parse(&input);
        assert_eq!(
            tree,
            ParseTree {
                value: Symbol::NonTerminal("S".to_string()),
                children: vec![ParseTree {
                    value: Symbol::NonTerminal("Identifier".to_string()),
                    children: vec![
                        ParseTree {
                            value: Symbol::NonTerminal("S".to_string()),
                            children: vec![ParseTree {
                                value: Symbol::NonTerminal("Identifier".to_string()),
                                children: vec![ParseTree {
                                    value: Symbol::Terminal("id".to_string()),
                                    children: vec![]
                                }]
                            }]
                        },
                        ParseTree {
                            value: Symbol::Terminal(".".to_string()),
                            children: vec![]
                        },
                        ParseTree {
                            value: Symbol::Terminal("id".to_string()),
                            children: vec![]
                        }
                    ]
                }]
            }
        )
    }

    #[test]
    fn test_parse_simple_method_call() {
        let cfg = get_test_cfg();
        let parser = Lr1Parser::new(&cfg);
        let input = ["id", "(", ")"];
        let tree = parser.parse(&input);
        assert_eq!(
            tree,
            ParseTree {
                value: Symbol::NonTerminal("S".to_string()),
                children: vec![ParseTree {
                    value: Symbol::NonTerminal("MethodCall".to_string()),
                    children: vec![
                        ParseTree {
                            value: Symbol::NonTerminal("Identifier".to_string()),
                            children: vec![ParseTree {
                                value: Symbol::Terminal("id".to_string()),
                                children: vec![]
                            }]
                        },
                        ParseTree {
                            value: Symbol::Terminal("(".to_string()),
                            children: vec![]
                        },
                        ParseTree {
                            value: Symbol::Terminal(")".to_string()),
                            children: vec![]
                        },
                    ]
                }]
            }
        )
    }

    #[test]
    fn test_parse_method_call() {
        let cfg = get_test_cfg();
        let parser = Lr1Parser::new(&cfg);
        let input = ["id", "(", ")", ".", "id", "(", ")"];
        let tree = parser.parse(&input);

        // parse tree for "id ( )"
        let simple_method_call = ParseTree {
            value: Symbol::NonTerminal("S".to_string()),
            children: vec![ParseTree {
                value: Symbol::NonTerminal("MethodCall".to_string()),
                children: vec![
                    ParseTree {
                        value: Symbol::NonTerminal("Identifier".to_string()),
                        children: vec![ParseTree {
                            value: Symbol::Terminal("id".to_string()),
                            children: vec![],
                        }],
                    },
                    ParseTree {
                        value: Symbol::Terminal("(".to_string()),
                        children: vec![],
                    },
                    ParseTree {
                        value: Symbol::Terminal(")".to_string()),
                        children: vec![],
                    },
                ],
            }],
        };

        assert_eq!(
            tree,
            ParseTree {
                value: Symbol::NonTerminal("S".to_string()),
                children: vec![ParseTree {
                    value: Symbol::NonTerminal("MethodCall".to_string()),
                    children: vec![
                        ParseTree {
                            value: Symbol::NonTerminal("Identifier".to_string()),
                            children: vec![
                                simple_method_call,
                                ParseTree {
                                    value: Symbol::Terminal(".".to_string()),
                                    children: vec![]
                                },
                                ParseTree {
                                    value: Symbol::Terminal("id".to_string()),
                                    children: vec![]
                                }
                            ]
                        },
                        ParseTree {
                            value: Symbol::Terminal("(".to_string()),
                            children: vec![]
                        },
                        ParseTree {
                            value: Symbol::Terminal(")".to_string()),
                            children: vec![]
                        },
                    ]
                }]
            }
        )
    }
}
