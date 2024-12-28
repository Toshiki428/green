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
<expression> ::= <logical> | <compare> | <add_and_sub> | <mul_and_div> | <unary> | <variable>
<logical> ::= <or_expr> | <and_expr> | <not_expr>
<or_expr> ::= <and_expr> ("or" <and_expr>)?
<and_expr> ::= <not_expr> ("and" <not_expr>)?
<not_expr> ::= ("not")? (<bool> | <compare> | "(" <logical> ")" )
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

演算の優先度
```
カッコ内 > 掛け算割り算 > 足し算引き算 > 比較演算 
> Not > and xor > or > 代入演算子
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
