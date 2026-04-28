Given a context free grammar, determine if the grammar is LR(1) and parse input.

```sh
cargo run -- <path_to_cfg> <path_to_input>
```

Prints LR(1) DFA and rightmost derivation.

**Example**

```sh
cargo run -- grammar/grammar.cfg examples/grammar.txt
```

```
State 0
E -> • E + E' : [ $, + ]
E -> • E' : [ $, + ]
E' -> • T : [ $, + ]
T -> • T * T' : [ $, *, + ]
T -> • T' : [ $, *, + ]
T' -> • F : [ $, *, + ]
F -> • ( E ) : [ $, *, + ]
F -> • id : [ $, *, + ]
S' -> • E $ : [ $ ]

State 1
E' -> T • : [ $, + ]
T -> T • * T' : [ $, *, + ]

State 2
T -> T' • : [ $, *, + ]

State 3
E -> E • + E' : [ $, + ]
S' -> E • $ : [ $ ]

State 4
T' -> F • : [ $, *, + ]

State 5
F -> id • : [ $, *, + ]

State 6
E -> • E + E' : [ ), + ]
E -> • E' : [ ), + ]
E' -> • T : [ ), + ]
T -> • T * T' : [ ), *, + ]
T -> • T' : [ ), *, + ]
T' -> • F : [ ), *, + ]
F -> • ( E ) : [ ), *, + ]
F -> ( • E ) : [ $, *, + ]
F -> • id : [ ), *, + ]

State 7
E -> E' • : [ $, + ]

State 8
T -> T * • T' : [ $, *, + ]
T' -> • F : [ $, *, + ]
F -> • ( E ) : [ $, *, + ]
F -> • id : [ $, *, + ]

State 9
E -> E + • E' : [ $, + ]
E' -> • T : [ $, + ]
T -> • T * T' : [ $, *, + ]
T -> • T' : [ $, *, + ]
T' -> • F : [ $, *, + ]
F -> • ( E ) : [ $, *, + ]
F -> • id : [ $, *, + ]

State 10
S' -> E $ • : [ $ ]

State 11
E' -> T • : [ ), + ]
T -> T • * T' : [ ), *, + ]

State 12
T' -> F • : [ ), *, + ]

State 13
T -> T' • : [ ), *, + ]

State 14
E -> E • + E' : [ ), + ]
F -> ( E • ) : [ $, *, + ]

State 15
E -> E' • : [ ), + ]

State 16
E -> • E + E' : [ ), + ]
E -> • E' : [ ), + ]
E' -> • T : [ ), + ]
T -> • T * T' : [ ), *, + ]
T -> • T' : [ ), *, + ]
T' -> • F : [ ), *, + ]
F -> • ( E ) : [ ), *, + ]
F -> ( • E ) : [ ), *, + ]
F -> • id : [ ), *, + ]

State 17
F -> id • : [ ), *, + ]

State 18
T -> T * T' • : [ $, *, + ]

State 19
E -> E + E' • : [ $, + ]

State 20
T -> T * • T' : [ ), *, + ]
T' -> • F : [ ), *, + ]
F -> • ( E ) : [ ), *, + ]
F -> • id : [ ), *, + ]

State 21
F -> ( E ) • : [ $, *, + ]

State 22
E -> E + • E' : [ ), + ]
E' -> • T : [ ), + ]
T -> • T * T' : [ ), *, + ]
T -> • T' : [ ), *, + ]
T' -> • F : [ ), *, + ]
F -> • ( E ) : [ ), *, + ]
F -> • id : [ ), *, + ]

State 23
E -> E • + E' : [ ), + ]
F -> ( E • ) : [ ), *, + ]

State 24
T -> T * T' • : [ ), *, + ]

State 25
E -> E + E' • : [ ), + ]

State 26
F -> ( E ) • : [ ), *, + ]

Rightmost derivation:
Apply E -> [NonTerminal("E"), Terminal("+"), NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("id")]
Apply E -> [NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T"), Terminal("*"), NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("("), NonTerminal("E"), Terminal(")")]
Apply E -> [NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("("), NonTerminal("E"), Terminal(")")]
Apply E -> [NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T"), Terminal("*"), NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("("), NonTerminal("E"), Terminal(")")]
Apply E -> [NonTerminal("E"), Terminal("+"), NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("id")]
Apply E -> [NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("id")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("id")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("("), NonTerminal("E"), Terminal(")")]
Apply E -> [NonTerminal("E"), Terminal("+"), NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("id")]
Apply E -> [NonTerminal("E"), Terminal("+"), NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("id")]
Apply E -> [NonTerminal("E'")]
Apply E' -> [NonTerminal("T")]
Apply T -> [NonTerminal("T'")]
Apply T' -> [NonTerminal("F")]
Apply F -> [Terminal("id")]
```
