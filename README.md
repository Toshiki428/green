# Green言語

## 構文
```txt
<program> :== <function_call> ";" | <program> <function_call> ";"
<function_call> ::= <function_name> "(" <argument> ")" 
<function_name> ::= "print"
<argument> = <expression> | <bool>
<expression> :== <compare>
<compare> :== <value> (("==" | "!=") <value>)
<value> :== <add_and_sub> | <string>
<add_and_sub> :== <mul_and_div> (("+" | "-") <mul_and_div>)*
<mul_and_div> :== <unary> (("*" | "/") <unary>)*
<unary> = <primary> | "-" <primary>
<primary> :== <number> | "(" <add_and_sub> ")"
<string> = "\"" [a-zA-Z0-9 ]* "\""
<number> :== [0-9]+
<bool> :== "true" | "false"
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
