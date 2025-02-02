use crate::common::{
    operator::*, types::{BlockType, LiteralValue, Type}
};

// -------------------------------------

/// ルートのノード
#[derive(Debug, PartialEq, Clone)]
pub struct RootNode {
    pub functions: Vec<FunctionDefinitionNode>,
    pub coroutines: Vec<CoroutineDefinitionNode>,
}


// -------------------------------------


/// 関数定義ノード（ルートノード直下）
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinitionNode {
    pub name: String,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<Type>,
    pub block: BlockNode,
    pub doc: Option<String>,
}

/// コルーチン定義ノード（ルートノード直下）
#[derive(Debug, PartialEq, Clone)]
pub struct CoroutineDefinitionNode {
    pub name: String,
    pub block: BlockNode,
    pub doc: Option<String>,
}

/// 関数のパラメータ定義ノード
#[derive(Debug, PartialEq, Clone)]
pub struct ParameterNode {
    pub name: String,
    pub variable_type: Type,
}


// -------------------------------------

/// 関数、コルーチン、ループ、条件分岐のブロックノード
#[derive(Debug, PartialEq, Clone)]
pub struct BlockNode {
    pub block_type: BlockType,
    pub statements: Vec<PrivateNode>,
}

/// 関数内のノード
#[derive(Debug, PartialEq, Clone)]
pub enum PrivateNode {
    /// 関数呼び出し
    FunctionCall { 
        name: String,
        arguments: Vec<Self>,
        return_flg: bool,
    },

    /// コルーチンのインスタンス化
    CoroutineInstantiation {
        task_name: String,
        coroutine_name: String,
    },

    /// コルーチンの再開
    CoroutineResume {
        task_name: String,
    },
    Yield,

    /// 変数宣言
    VariableDeclaration {
        name: String,
        variable_type: Type,
        initializer: Option<Box<Self>>,
        doc: Option<String>,
    },
    /// 変数代入
    VariableAssignment {
        name: String,
        expression: Box<Self>,
    },
    /// 変数呼び出し
    Variable {
        name: String,
    },

    /// If文
    IfStatement {
        condition_node: Box<Self>,
        then_block: BlockNode,
        else_block: Option<BlockNode>,
    },
    /// ループ文
    LoopStatement {
        condition_node: Box<Self>,
        block: BlockNode,
    },

    /// return文
    ReturnStatement {
        assignalbe: Box<Self>,
    },

    ProcessComment {
        comment: String,
    },

    Break,
    Continue,
    Error,
    
    // --------------------

    /// 論理演算
    Logical {
        operator: Logical,
        left: Box<Self>,
        right: Option<Box<Self>>,
    },
    /// 比較演算
    Compare {
        operator: Comparison,
        left: Box<Self>,
        right: Box<Self>,
    },
    /// 算術演算
    Arithmetic {
        operator: Arithmetic,
        left: Box<Self>,
        right: Option<Box<Self>>,
    },
    /// リテラル値
    Literal {
        value: LiteralValue,
    },
}
