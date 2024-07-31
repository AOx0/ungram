# Ungrammar

Implementation of the [Ungrammar][1] formalism for describing concrete syntax trees.

This CLI tool provides:
- parser for Ungrammar files (`.ungram`)
- FIRST set calculator
- FOLLOW set calculator

# Example

```py
# example.ungram
S = File '#'
File = Fn*

Fn = 'fn' 'name' ParamList ('->' 'type')? Block

ParamList = '(' Param* ')'
Param = 'name' ':' 'type' ','?
Block = '{' 'statements' '}'
```

## FIRST set

```sh
ungram first example.ungram
```

Output:
```py
S: {"fn", "ε"}
File: {"fn", "ε"}
Fn: {"fn"}
ParamList: {"("}
Param: {"name"}
Block: {"{"}
```

## FOLLOW set

```sh
ungram follow example.ungram
```

Output:
```py
S: {}
File: {"#"}
Fn: {"fn", "#"}
ParamList: {"{", "->"}
Param: {"name", ")"}
Block: {"fn", "#"}
```

[1]: https://rust-analyzer.github.io/blog/2020/10/24/introducing-ungrammar.html
