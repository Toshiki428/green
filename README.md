# Green言語

## 基本型
- `int`：整数
- `float`：浮動小数点
- `string`：文字列
- `bool`：真偽値

## 基本構文

### 変数定義
```grn
let a: int = 10;
```

### 条件分岐
```grn
let a : int = 10;
if (a >= 10 and a < 20) {
  print("OK");
}
else {
  print("NG");
}
```

### ループ
```grn
let i: int = 0;
while (i < 10) {
  print("a");
  i = i+1;
}
```

### 関数定義
```grn
// 戻り値なしの場合
function print_sum(first: int, second: int) {
  print(first + second);
}

// 戻り値ありの場合
function sum(first: int, second: int) -> int {
  return first + second;
}
```

### コルーチン定義
```
coroutine print_alpha() {
  print("A");
  yield;
  print("B");
  yield;
  print("C");
}

coro task = print_alpha();
resume task;
resume task;
resume task;
```

### コメント
```grn
// 行コメント
/*
ブロックコメント
*/
```

- `Docコメント`
  - 変数、関数、コルーチン定義の前に書いたコメントが情報として保存される
  - `@process`と頭につけたDocコメントブロックは、シーケンス図上に表示することができる

```grn
/// カウント用の変数
let count: int = 0;
```

```grn
/// @process 10回ループ
```

## ToDo
- 配列を追加
- forループを追加
- 型推論
