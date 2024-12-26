# Green言語

## 構文
```txt
<program> ::= (<statement>)*
<statement> ::= (<function_call> | <variable_declaration>) ";"
<function_call> ::= <function_name> "(" <argument> ")"
<function_name> ::= "print"
<argument> ::= <assignable>
<variable_declaration> ::= "let " <variable> "=" <assignable>
<assignable> ::= <expression> | <literal>
<literal> ::= <bool> | <string> | <number>
<expression> ::= <compare> | <add_and_sub> | <mul_and_div> | <unary> | <variable>
<compare> ::= <value> (("==" | "!=" | ">=" | "<=" | ">" | "<") <value>)?
<value> ::= <add_and_sub> | <string>
<add_and_sub> ::= <mul_and_div> (("+" | "-") <mul_and_div>)*
<mul_and_div> ::= <unary> (("*" | "/") <unary>)*
<unary> ::= <primary> | "-" <primary>
<primary> ::= <number> | "(" <add_and_sub> ")" | <variable>
<variable> ::= [a-zA-Z_][a-zA-Z0-9_]*
<string> ::= "\"" [a-zA-Z0-9 ]* "\""
<number> ::= [0-9]+
<bool> ::= "true" | "false"
```

## エラーコード
```
[カテゴリコード][番号]
```

- `[カテゴリコード]`
  - CMD
  - FILE
  - SYNTAX
  - RUNTIME
  - ALL
