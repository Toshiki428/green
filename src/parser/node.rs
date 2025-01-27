use crate::common::{
    operator::*,
    keyword::TypeName,
    types::{BlockType, LiteralValue},
};

/// ルートのノード
#[derive(Debug, PartialEq, Clone)]
pub enum RootNode {
    Program {
        functions: Vec<GlobalNode>,
        coroutines: Vec<GlobalNode>,
    },
}

impl RootNode {
    /// デバッグ用のprint文
    pub fn print(&self, depth: i32) {
        self.indent(depth);
        match self {
            Self::Program { functions, coroutines } => {
                println!("functions:");
                for function in functions {
                    function.print(depth+1);
                }
                self.indent(depth);
                println!("coroutines:");
                for coroutine in coroutines {
                    coroutine.print(depth+1);
                }
            },
        }
    }

    fn indent(&self, depth: i32) {
        for _ in 0..depth {
            print!("  ");
        }
    }
}

/// ルートノード直下のノード
#[derive(Debug, PartialEq, Clone)]
pub enum GlobalNode {
    /// 関数定義
    FunctionDefinition {
        name: String,
        parameters: Vec<Self>,
        block: PrivateNode,
        doc: Option<String>,
    },
    /// コルーチン定義
    CoroutineDefinition {
        name: String,
        block: PrivateNode,
        doc: Option<String>,
    },
    /// 引数
    Parameter {
        name: String,
        variable_type: TypeName,
    },
}
impl GlobalNode {
    /// デバッグ用のprint文
    pub fn print(&self, depth: i32) {
        self.indent(depth);
        match self {
            Self::FunctionDefinition { name, parameters, block, doc } => {
                println!("FunctionDefinition: {}", name);
                if let Some(comment) = doc {
                    self.indent(depth+1);
                    println!("DocComment: {}", comment);
                }
                
                block.print(depth+1);
                for param in parameters {
                    param.print(depth+1);
                }
            },
            Self::CoroutineDefinition { name, block, doc } => {
                println!("CoroutineDefinition: {}", name);
                if let Some(comment) = doc {
                    self.indent(depth+1);
                    println!("DocComment: {}", comment);
                }

                block.print(depth+1);
            },
            Self::Parameter { name, variable_type } => {
                println!("{}: {}", name, variable_type.to_string())
            },
        }
    }

    fn indent(&self, depth: i32) {
        for _ in 0..depth {
            print!("  ");
        }
    }
}

/// 関数内のノード
#[derive(Debug, PartialEq, Clone)]
pub enum PrivateNode {
    /// ブロックのノード（関数、コルーチン、ループ、条件分岐）
    Block {
        block_type: BlockType,
        statements: Vec<Self>,
    },

    /// 関数呼び出し（戻り値なし）
    FunctionCall { 
        name: String,
        arguments: Vec<Self>,
    },
    /// 関数呼び出し（戻り値あり）
    FunctionCallWithReturn {
        name: String,
        arguments: Vec<Self>,
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
        variable_type: TypeName,
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
        then_block: Box<Self>,
        else_block: Option<Box<Self>>,
    },
    /// ループ文
    LoopStatement {
        condition_node: Box<Self>,
        block: Box<Self>,
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
impl PrivateNode {
    /// デバッグ用のprint文
    pub fn print(&self, depth: i32) {
        self.indent(depth);
        match self {
            Self::Block { block_type, statements } => {
                println!("block: ({})", block_type.to_string());
                for statement in statements {
                    statement.print(depth+1);
                }
            },
            Self::FunctionCall { name, arguments } => {
                println!("FunctionCall: {}", name);
                self.indent(depth+1);
                println!("Args:");
                for argument in arguments {
                    argument.print(depth+2);
                }
            },
            Self::FunctionCallWithReturn { name, arguments } => {
                println!("FunctionCallWithReturn: {}", name);
                self.indent(depth+1);
                println!("Args:");
                for argument in arguments {
                    argument.print(depth+2);
                }
            },
            Self::CoroutineResume { task_name  } => {
                println!("CoroutineResume: {}", task_name);
            },
            Self::CoroutineInstantiation { task_name, coroutine_name } => {
                println!("CoroutineInstantiation: {} -> {}", coroutine_name, task_name);
            },
            Self::Yield => {
                println!("Yield");
            },
            Self::VariableDeclaration { name, variable_type, initializer, doc } => {
                println!("VariableDeaclaration: {} ({})", name, variable_type.to_string());
                if let Some(comment) = doc {
                    self.indent(depth+1);
                    println!("DocComment: {}", comment);
                }

                if let Some(expression) = initializer {
                    self.indent(depth+1);
                    println!("initializer:");
                    expression.print(depth+2);
                }
            },
            Self::VariableAssignment { name, expression } => {
                println!("VariableAssignment: {}", name);
                expression.print(depth+1);
            },
            Self::Variable { name } => {
                println!("Variable: {}", name);
            },
            Self::Logical { operator, left, right } => {
                println!("operator: {}", operator.to_string());
                self.indent(depth+1);
                println!("left:");
                left.print(depth+2);
                if let Some(node) = right {
                    self.indent(depth+1);
                    println!("right:");
                    node.print(depth+2);
                }
            },
            Self::Compare { operator, left, right } => {
                println!("operator: {}", operator.to_string());
                self.indent(depth+1);
                println!("left:");
                left.print(depth+2);
                self.indent(depth+1);
                println!("right:");
                right.print(depth+2);
            },
            Self::Arithmetic { operator, left, right } => {
                println!("operator: {}", operator.to_string());
                self.indent(depth+1);
                println!("left:");
                left.print(depth+2);
                if let Some(node) = right {
                    self.indent(depth+1);
                    println!("right:");
                    node.print(depth+2);
                }
            },
            Self::Literal { value } => println!("Literal: {}", value.to_string()),
            Self::IfStatement { condition_node, then_block, else_block } => {
                println!("IfStatement:");
                self.indent(depth+1);
                println!("condition_node");
                condition_node.print(depth+2);
                self.indent(depth+1);
                println!("then_block");
                then_block.print(depth+2);
                if let Some(node) = else_block {
                    self.indent(depth+1);
                    println!("else_block");
                    node.print(depth+2);
                }
            },
            Self::LoopStatement { condition_node, block } => {
                println!("LoopStatement:");
                self.indent(depth+1);
                println!("condition_node");
                condition_node.print(depth+2);
                block.print(depth+2);
            },
            Self::ReturnStatement { assignalbe } => {
                println!("Return:");
                assignalbe.print(depth+1);
            },

            Self::ProcessComment { comment } => {
                println!("ProcessComment: {}", comment);
            }

            Self::Break => println!("break"),
            Self::Continue => println!("continue"),
            Self::Error => println!("Error"),
        }
    }

    fn indent(&self, depth: i32) {
        for _ in 0..depth {
            print!("  ");
        }
    }
}