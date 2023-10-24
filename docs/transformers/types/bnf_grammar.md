#

## `bnf_grammar`

This logits transformer will shape the logits input following a defined grammar in BNF form, making the model only generates tokens in your expected format. For example:

```text
dna := 'A' | 'T' | 'C' | 'G' | 'N'
dna_sequence := <dna> | <dna><dna_sequence>
start := <dna_sequence>'*'
```

Will force the pipeline to generate only `ATCGN` until any `*` is generated. A detailed explanation of each grammar is as followed:

```text
dna := 'A' | 'T' | 'C' | 'G' | 'N'
```

Defines a `non-terminal` which matches any *single* character in the `ATCGN`. `Terminals` are things appearing on the right - they define a set of things to match.

Things on the left of `:=` are `non-terminal`s. It is the symbol of a concrete grammar that matches something. The logits transformer starts by a `non-terminal`, and also you can use `non-terminal`s in `terminal`s to reduce repetitive patterns.

```text
dna_sequence := <dna> | <dna><dna_sequence>
```

Defines a `non-terminal` which matches either a single `dna`, or a `dna` followed by itself. Since each `dna_sequence` can be interpreted into a `<dna><dna_sequence>`, making it a recursive non-terminal, so it will match `dna` for infinite time.

```text
start := <dna_sequence>'*'
```

Defines a `non-terminal` consists of the `dna_sequence` and the `*`. The recursive `non-terminal` will not end itself, but adding a exhaustive (meaning that it can complete) node after it will make the matches in the exhaustive node too.

So, by adding `*` after `dna_sequence`, it makes `*` a choice other than `ATCGN`s. And when the grammar matches `*`, the recursion is exited and continue to match the `*`. And by then, the matcher will be able to reach the end and stop.

#### Params
