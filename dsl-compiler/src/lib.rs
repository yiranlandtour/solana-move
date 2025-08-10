use pest::Parser;
use pest_derive::Parser;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[grammar = "../grammar.pest"]
pub struct DslParser;

pub mod codegen;
pub mod parser;
pub mod semantic;
pub mod optimizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub state: Vec<StateVariable>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVariable {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub visibility: Visibility,
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    U8,
    U64,
    U128,
    Bool,
    Address,
    String,
    Map(Box<Type>, Box<Type>),
    Vec(Box<Type>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Let { name: String, value: Expression },
    Assign { target: String, value: Expression },
    If { condition: Expression, then_block: Vec<Statement>, else_block: Option<Vec<Statement>> },
    Require { condition: Expression, message: Option<String> },
    Emit { event: String, args: Vec<Expression> },
    Return { value: Option<Expression> },
    Expression(Expression),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Number(u64),
    Bool(bool),
    String(String),
    Identifier(String),
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    Unary { op: UnaryOp, expr: Box<Expression> },
    Call { func: String, args: Vec<Expression> },
    Index { array: String, index: Box<Expression> },
    Field { object: String, field: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Not, Neg,
}

impl Contract {
    pub fn parse(input: &str) -> Result<Self> {
        let pairs = DslParser::parse(Rule::program, input)
            .map_err(|e| anyhow!("Parse error: {}", e))?;
        
        // 简化的解析实现（实际需要完整实现）
        Ok(Contract {
            name: "Token".to_string(),
            state: vec![],
            functions: vec![],
        })
    }
}