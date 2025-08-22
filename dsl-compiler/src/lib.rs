use pest::Parser;
use pest_derive::Parser;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "../grammar.pest"]
pub struct DslParser;

pub mod codegen;
pub mod parser;
pub mod semantic;
pub mod semantic_analyzer;
pub mod optimizer;

pub use semantic_analyzer::{SemanticAnalyzer, SymbolTable, TypeInference};

// Enhanced AST definitions with more comprehensive node types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub contracts: Vec<Contract>,
    pub imports: Vec<Import>,
    pub type_definitions: Vec<TypeDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub state: Vec<StateVariable>,
    pub structs: Vec<StructDefinition>,
    pub functions: Vec<Function>,
    pub events: Vec<EventDefinition>,
    pub modifiers: Vec<Modifier>,
    pub constants: Vec<Constant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    pub name: String,
    pub params: Vec<EventParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventParam {
    pub name: String,
    pub ty: Type,
    pub indexed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constant {
    pub name: String,
    pub ty: Type,
    pub value: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVariable {
    pub name: String,
    pub ty: Type,
    pub visibility: Visibility,
    pub is_mutable: bool,
    pub initial_value: Option<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub visibility: Visibility,
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub modifiers: Vec<String>,
    pub body: Vec<Statement>,
    pub is_payable: bool,
    pub is_view: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Internal,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    I8,
    I16,
    I32,
    I64,
    I128,
    Bool,
    Address,
    String,
    Bytes,
    Map(Box<Type>, Box<Type>),
    Vec(Box<Type>),
    Array(Box<Type>, usize),
    Tuple(Vec<Type>),
    Struct(String),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Let { 
        name: String, 
        ty: Option<Type>,
        value: Expression,
        is_mutable: bool,
    },
    Assign { 
        target: LValue, 
        value: Expression 
    },
    If { 
        condition: Expression, 
        then_block: Vec<Statement>, 
        else_block: Option<Vec<Statement>> 
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        init: Box<Statement>,
        condition: Expression,
        update: Box<Statement>,
        body: Vec<Statement>,
    },
    ForEach {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Require { 
        condition: Expression, 
        message: Option<String> 
    },
    Assert {
        condition: Expression,
        message: Option<String>,
    },
    Emit { 
        event: String, 
        args: Vec<Expression> 
    },
    Return { 
        value: Option<Expression> 
    },
    Break,
    Continue,
    Expression(Expression),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LValue {
    Identifier(String),
    Index { 
        array: Box<LValue>, 
        index: Box<Expression> 
    },
    Field { 
        object: Box<LValue>, 
        field: String 
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Number(u64),
    Float(f64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    Identifier(String),
    Binary { 
        op: BinaryOp, 
        left: Box<Expression>, 
        right: Box<Expression> 
    },
    Unary { 
        op: UnaryOp, 
        expr: Box<Expression> 
    },
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    Call { 
        func: Box<Expression>, 
        args: Vec<Expression> 
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    Index { 
        array: Box<Expression>, 
        index: Box<Expression> 
    },
    Field { 
        object: Box<Expression>, 
        field: String 
    },
    ArrayLiteral(Vec<Expression>),
    TupleLiteral(Vec<Expression>),
    StructLiteral {
        name: String,
        fields: HashMap<String, Expression>,
    },
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expression>,
    },
    MsgSender,
    MsgValue,
    BlockNumber,
    BlockTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,
    // Comparison
    Eq, Ne, Lt, Gt, Le, Ge,
    // Logical
    And, Or,
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Not, Neg, BitNot,
}

impl Contract {
    pub fn parse(input: &str) -> Result<Self> {
        let pairs = DslParser::parse(Rule::program, input)
            .map_err(|e| anyhow!("Parse error: {}", e))?;
        
        // Call the actual parser implementation
        parser::parse_contract_from_pairs(pairs)
    }
}