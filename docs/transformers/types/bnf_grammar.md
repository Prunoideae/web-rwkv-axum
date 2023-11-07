#

## `bnf_grammar`

This logits transformer will shape the logits input following a defined grammar in BNF form, making the model only generates tokens in your expected format. For example:

```BNF
<dna> ::= 'A' | 'T' | 'C' | 'G' | 'N'
<dna_sequence> ::= <dna> | <dna><dna_sequence>
<start> ::= <dna_sequence>'*'
```

Will force the pipeline to generate only `ATCGN` until any `*` is generated. A detailed explanation of each grammar is as followed:

```BNF
<dna> ::= 'A' | 'T' | 'C' | 'G' | 'N'
```

Defines a `non-terminal` which matches any *single* character in the `ATCGN`. `Terminals` are things appearing on the right - they define a set of things to match.

Things on the left of `::=` are `non-terminal`s. It is the symbol of a concrete grammar that matches something. The logits transformer starts by a `non-terminal`, and also you can use `non-terminal`s in `terminal`s to reduce repetitive patterns.

```BNF
<dna_sequence> ::= <dna> | <dna><dna_sequence>
```

Defines a `non-terminal` which matches either a single `dna`, or a `dna` followed by itself. Since each `dna_sequence` can be interpreted into a `<dna><dna_sequence>`, making it a recursive non-terminal, so it will match `dna` for infinite time.

```BNF
<start> ::= <dna_sequence>'*'
```

Defines a `non-terminal` consists of the `dna_sequence` and the `*`. The recursive `non-terminal` will not end itself, but adding a exhaustive (meaning that it can complete) node after it will make the matches in the exhaustive node too.

So, by adding `*` after `dna_sequence`, it makes `*` a choice other than `ATCGN`s. And when the grammar matches `*`, the recursion is exited and continue to match the `*`. And by then, the matcher will be able to reach the end and stop.

Check more examples and specification [here](https://github.com/Dan-wanna-M/bnf_sampler#bnf_sampler).

#### Params

```json
{
    "grammar": "<sequence>::=<any!>|<any!><sequence>\n<start>::=<sequence>",
    // a grammar that do not reject any token. Only for demonstration purpose.
    "stack_arena_capacity": 1048576, 
    // increase stack_arena_capacity if you see an error message telling you to increase it.
    // This means some tokens match a lot of consecutive terminals and require more spaces.
    "grammar_stack_arena_capacity": 1024, 
    // increase grammar_stack_arena_capacity if you see an error message telling you to increase it.
    // This means some nonterminal used in except!() expand to a lot of intermediate nonterminals.
    "start_nonterminal": "start",
    // start_nonterminal specifies the initial nonterminal(entry point) of BNF grammar
    "stack_to_bytes_cache_enabled": true,
    // This usually improves performance by caching previous states. 
}
```
