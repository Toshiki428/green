#[derive(Debug, PartialEq, Clone)]
pub enum ErrorCode {
    /// コマンドオプションエラー
    Cmd001,
    /// コマンド構文エラー
    Cmd002,

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
    
    Runtime001,
    Runtime002,
    Runtime003,
    Runtime004,
    Runtime005,
    Runtime006,
    Runtime007,
    Runtime008,
    Runtime009,
    Runtime010,
    Runtime011,
    Runtime012,
    Runtime013,
    Runtime014,
    Runtime015,
    Runtime016,
    Runtime017,
    Runtime018,
    ALL,
}

impl ErrorCode {
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::Cmd001 => "CMD001",
            Self::Cmd002 => "CMD002",
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
            Self::ALL => "ALL",
        };
        str.to_string()
    }
}