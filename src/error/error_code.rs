#[derive(Debug, PartialEq, Clone)]
pub enum ErrorCode {
    /// ファイルが見つからない
    Io001,

    /// 字句エラー
    Lex001,
    /// 想定外文字
    Lex002,
    /// 文字列終端
    Lex003,
    /// 想定外の終了
    Lex004,
    /// 未定義の演算子
    Lex005,

    /// 構文エラー
    Parse001,
    /// 想定外トークン
    Parse002,
    /// 想定外の終了
    Parse003,
    /// 想定外の数値
    Parse004,
    /// 特定のトークン不足
    Parse005,
    /// 特定ブロック内でのみ使えるキーワード
    Parse006,

    /// 意味解析エラー
    Semantic001,
    /// 異なる型の演算
    Semantic002,
    /// 想定外のノード
    Semantic003,
    /// 定義されていない関数の呼び出し
    Semantic004,
    /// 戻り値の型が定義されていない
    Semantic005,
    /// 不正な変数代入
    Semantic006,
    /// 定義されていない変数の呼び出し
    Semantic007,
    /// 引数の個数の不一致
    Semantic008,
    
    /// 実行時エラー
    Runtime001,
    /// 未定義の関数呼び出し
    Runtime002,
    /// 想定外ノード
    Runtime003,
    Runtime004,
    /// 想定外の値
    Runtime005,
    /// 想定外の文字列比較
    Runtime006,
    /// 未定義の変数呼び出し
    Runtime007,
    /// 想定外の演算
    Runtime008,
    /// 代入値不足
    Runtime009,
    /// 型エラー
    Runtime010,
    /// 想定外の引数
    Runtime011,
    /// 引数の個数
    Runtime012,
    /// 引数の型エラー
    Runtime013,
    /// 想定外のif条件
    Runtime014,
    /// 想定外の演算
    Runtime015,
    /// 異なる型の比較
    Runtime016,
    /// 想定外のループ条件
    Runtime017,
    /// 想定外のフロー
    Runtime018,
    /// 未定義のコルーチン
    Runtime019,
    /// 完了したタスクの呼び出し
    Runtime020,

    ALL,
}

impl ErrorCode {
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::Io001 => "IO001",
            Self::Lex001 => "LEX001",
            Self::Lex002 => "LEX002",
            Self::Lex003 => "LEX003",
            Self::Lex004 => "LEX004",
            Self::Lex005 => "LEX005",
            Self::Parse001 => "PARSE001",
            Self::Parse002 => "PARSE002",
            Self::Parse003 => "PARSE003",
            Self::Parse004 => "PARSE004",
            Self::Parse005 => "PARSE005",
            Self::Parse006 => "PARSE006",
            Self::Semantic001 => "SEMANTIC001",
            Self::Semantic002 => "SEMANTIC002",
            Self::Semantic003 => "SEMANTIC003",
            Self::Semantic004 => "SEMANTIC004",
            Self::Semantic005 => "SEMANTIC005",
            Self::Semantic006 => "SEMANTIC006",
            Self::Semantic007 => "SEMANTIC007",
            Self::Semantic008 => "SEMANTIC008",
            Self::Runtime001 => "RUNTIME001",
            Self::Runtime002 => "RUNTIME002",
            Self::Runtime003 => "RUNTIME003",
            Self::Runtime004 => "RUNTIME004",
            Self::Runtime005 => "RUNTIME005",
            Self::Runtime006 => "RUNTIME006",
            Self::Runtime007 => "RUNTIME007",
            Self::Runtime008 => "RUNTIME008",
            Self::Runtime009 => "RUNTIME009",
            Self::Runtime010 => "RUNTIME010",
            Self::Runtime011 => "RUNTIME011",
            Self::Runtime012 => "RUNTIME012",
            Self::Runtime013 => "RUNTIME013",
            Self::Runtime014 => "RUNTIME014",
            Self::Runtime015 => "RUNTIME015",
            Self::Runtime016 => "RUNTIME016",
            Self::Runtime017 => "RUNTIME017",
            Self::Runtime018 => "RUNTIME018",
            Self::Runtime019 => "RUNTIME019",
            Self::Runtime020 => "RUNTIME020",
            Self::ALL => "ALL",
        };
        str.to_string()
    }
}