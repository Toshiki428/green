# Green言語

## To Do
- 型チェックを入れる（今は`int`変数に`string`などが入る）

## 構文
```txt
<program> ::= <statements>
<statements> ::= <statement> | <statements> <statement>
<statement> ::= <function_call> | <variable_declaration> | <if_statement> | <function_definition> | <while_statement> | <coroutine_statement>
<if_statement> ::= "if" "(" <assignable> ")" <block> [ "else" <block> ]
<function_definition> = "function" <function_name> "(" ((<variable> ":" <type> "," )* <variable> ":" <type>)? ")" <function_block>
<while_statement> ::= "while" "(" <assignable> ")" <loop_block>
<coroutine_statement> ::= "coroutine" <function_name> "(" ")" <coroutine_block>
<block> ::= "{" <statements> "}"
<coroutine_block> ::= "{" <statements> "yield" "}"
<function_block> ::= "{" <statements> ("return" <assignable> ";")* "}"
<loop_block> ::= "{" <statements> ("continue" ";" | "break" ";")* "}"
<function_call> ::= <function_name> "(" <argument> ")"
<argument> ::= <assignable>
<variable_declaration> ::= "let " <variable> ":" <type> "=" <assignable>
<type> ::= "int" | "float" | "string" | "bool"
<assignable> ::= <expression> | <literal> | <function_call>
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
<function_name> ::= [a-zA-Z_][a-zA-Z0-9_]*
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

## コルーチンの書き方
```
coroutine name() {
  print("A");
  yield;
  print("B");
  yield;
  print("C");
}

coro task = name();
print(1);
resume task;
print(2);
resume task;
print(3);
resume task;
print(4);
```

出力結果
```
1
A
2
B
3
C
4
```