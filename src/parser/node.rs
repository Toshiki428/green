use crate::common::{
    operator::*,
    types::{BlockType, LiteralValue, Type},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    /// プログラム全体のノード
    Block {
        block_type: BlockType,
        statements: Vec<Node>,
    },
    /// 関数呼び出し（戻り値なし）
    FunctionCall { 
        name: String,
        arguments: Vec<Node>,
    },
    /// 関数呼び出し（戻り値あり）
    FunctionCallWithReturn {
        name: String,
        arguments: Vec<Node>,
    },
    /// 変数宣言
    VariableDeclaration {
        name: String,
        variable_type: Type,
    },
    /// 変数代入
    VariableAssignment {
        name: String,
        expression: Box<Node>,
    },
    /// 変数呼び出し
    Variable {
        name: String,
    },
    /// 論理演算
    Logical {
        operator: Logical,
        left: Box<Node>,
        right: Option<Box<Node>>,
    },
    /// 比較演算
    Compare {
        operator: Comparison,
        left: Box<Node>,
        right: Box<Node>,
    },
    /// 算術演算
    Arithmetic {
        operator: Arithmetic,
        left: Box<Node>,
        right: Option<Box<Node>>,
    },
    /// リテラル値
    Literal {
        value: LiteralValue,
    },
    /// If文
    IfStatement {
        condition_node: Box<Node>,
        then_block: Box<Node>,
        else_block: Option<Box<Node>>,
    },
    /// ループ文
    LoopStatement {
        condition_node: Box<Node>,
        block: Box<Node>,
    },
    /// 関数定義
    FunctionDefinition {
        name: String,
        parameters: Vec<Node>,
        block: Box<Node>,
    },
    ReturnStatement {
        assignalbe: Box<Node>,
    },
}

impl Node {
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
            Self::VariableDeclaration { name, variable_type } => {
                println!("VariableDeaclaration: {} ({})", name, variable_type.to_string());
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
                self.indent(depth+1);
                println!("block");
                block.print(depth+2);
            },
            Self::FunctionDefinition { name, parameters, block } => {
                println!("FunctionDefinition: {}", name);
                for param in parameters {
                    param.print(depth+1);
                }
                self.indent(depth+1);
                println!("block:");
                block.print(depth+2);
            },
            Self::ReturnStatement { assignalbe } => {
                println!("Return:");
                assignalbe.print(depth+1);
            }
        }
    }

    fn indent(&self, depth: i32) {
        for _ in 0..depth {
            print!("  ");
        }
    }
}