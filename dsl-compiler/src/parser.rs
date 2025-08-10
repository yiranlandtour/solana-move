use pest::iterators::{Pair, Pairs};
use pest::Parser;
use anyhow::{Result, anyhow, Context};
use crate::{DslParser, Rule, Contract, StateVariable, Function, Visibility, Parameter, Type, Statement, Expression, BinaryOp, UnaryOp};

impl Contract {
    pub fn parse(input: &str) -> Result<Self> {
        let pairs = DslParser::parse(Rule::program, input)
            .map_err(|e| anyhow!("Parse error: {}", e))?;
        
        let pair = pairs.into_iter().next()
            .ok_or_else(|| anyhow!("No program found"))?;
        
        parse_contract(pair.into_inner().next().unwrap())
    }
}

fn parse_contract(pair: Pair<Rule>) -> Result<Contract> {
    let mut inner = pair.into_inner();
    
    // Skip "contract" keyword and get name
    let name = inner.next()
        .ok_or_else(|| anyhow!("Missing contract name"))?
        .as_str()
        .to_string();
    
    let mut state = Vec::new();
    let mut functions = Vec::new();
    
    for item in inner {
        match item.as_rule() {
            Rule::state_section => {
                state = parse_state_section(item)?;
            }
            Rule::function_section => {
                for func_pair in item.into_inner() {
                    if func_pair.as_rule() == Rule::function_def {
                        functions.push(parse_function(func_pair)?);
                    }
                }
            }
            _ => {}
        }
    }
    
    Ok(Contract { name, state, functions })
}

fn parse_state_section(pair: Pair<Rule>) -> Result<Vec<StateVariable>> {
    let mut vars = Vec::new();
    
    for item in pair.into_inner() {
        if item.as_rule() == Rule::state_var {
            let mut inner = item.into_inner();
            
            let name = inner.next()
                .ok_or_else(|| anyhow!("Missing state variable name"))?
                .as_str()
                .to_string();
            
            let ty = parse_type(inner.next()
                .ok_or_else(|| anyhow!("Missing state variable type"))?)?;
            
            vars.push(StateVariable { name, ty });
        }
    }
    
    Ok(vars)
}

fn parse_function(pair: Pair<Rule>) -> Result<Function> {
    let mut inner = pair.into_inner();
    
    // Parse visibility
    let mut visibility = Visibility::Private;
    let mut current = inner.next().ok_or_else(|| anyhow!("Empty function"))?;
    
    if current.as_rule() == Rule::visibility {
        visibility = match current.as_str() {
            "public" => Visibility::Public,
            _ => Visibility::Private,
        };
        current = inner.next().ok_or_else(|| anyhow!("Missing function name"))?;
    }
    
    // Parse function name
    let name = current.as_str().to_string();
    
    // Parse parameters
    let mut params = Vec::new();
    if let Some(param_list) = inner.next() {
        if param_list.as_rule() == Rule::param_list {
            params = parse_param_list(param_list)?;
        }
    }
    
    // Parse return type
    let mut return_type = None;
    let mut body_pair = None;
    
    for item in inner {
        match item.as_rule() {
            Rule::type_spec => {
                return_type = Some(parse_type(item)?);
            }
            Rule::block => {
                body_pair = Some(item);
            }
            _ => {}
        }
    }
    
    // Parse body
    let body = if let Some(block) = body_pair {
        parse_block(block)?
    } else {
        Vec::new()
    };
    
    Ok(Function {
        visibility,
        name,
        params,
        return_type,
        body,
    })
}

fn parse_param_list(pair: Pair<Rule>) -> Result<Vec<Parameter>> {
    let mut params = Vec::new();
    
    for param_pair in pair.into_inner() {
        if param_pair.as_rule() == Rule::param {
            let mut inner = param_pair.into_inner();
            
            let name = inner.next()
                .ok_or_else(|| anyhow!("Missing parameter name"))?
                .as_str()
                .to_string();
            
            let ty = parse_type(inner.next()
                .ok_or_else(|| anyhow!("Missing parameter type"))?)?;
            
            params.push(Parameter { name, ty });
        }
    }
    
    Ok(params)
}

fn parse_type(pair: Pair<Rule>) -> Result<Type> {
    let type_str = pair.as_str();
    let mut inner = pair.into_inner();
    
    match type_str {
        "u8" => Ok(Type::U8),
        "u64" => Ok(Type::U64),
        "u128" => Ok(Type::U128),
        "bool" => Ok(Type::Bool),
        "address" => Ok(Type::Address),
        "string" => Ok(Type::String),
        _ if type_str.starts_with("map") => {
            let key_type = parse_type(inner.next()
                .ok_or_else(|| anyhow!("Missing map key type"))?)?;
            let value_type = parse_type(inner.next()
                .ok_or_else(|| anyhow!("Missing map value type"))?)?;
            Ok(Type::Map(Box::new(key_type), Box::new(value_type)))
        }
        _ if type_str.starts_with("vec") => {
            let elem_type = parse_type(inner.next()
                .ok_or_else(|| anyhow!("Missing vec element type"))?)?;
            Ok(Type::Vec(Box::new(elem_type)))
        }
        _ => Err(anyhow!("Unknown type: {}", type_str))
    }
}

fn parse_block(pair: Pair<Rule>) -> Result<Vec<Statement>> {
    let mut statements = Vec::new();
    
    for stmt_pair in pair.into_inner() {
        if stmt_pair.as_rule() == Rule::statement {
            statements.push(parse_statement(stmt_pair)?);
        }
    }
    
    Ok(statements)
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement> {
    let inner = pair.into_inner().next()
        .ok_or_else(|| anyhow!("Empty statement"))?;
    
    match inner.as_rule() {
        Rule::let_stmt => parse_let_stmt(inner),
        Rule::assign_stmt => parse_assign_stmt(inner),
        Rule::if_stmt => parse_if_stmt(inner),
        Rule::require_stmt => parse_require_stmt(inner),
        Rule::emit_stmt => parse_emit_stmt(inner),
        Rule::return_stmt => parse_return_stmt(inner),
        Rule::expr_stmt => {
            let expr = parse_expression(inner.into_inner().next().unwrap())?;
            Ok(Statement::Expression(expr))
        }
        _ => Err(anyhow!("Unknown statement type"))
    }
}

fn parse_let_stmt(pair: Pair<Rule>) -> Result<Statement> {
    let mut inner = pair.into_inner();
    
    let name = inner.next()
        .ok_or_else(|| anyhow!("Missing variable name"))?
        .as_str()
        .to_string();
    
    let value = parse_expression(inner.next()
        .ok_or_else(|| anyhow!("Missing variable value"))?)?;
    
    Ok(Statement::Let { name, value })
}

fn parse_assign_stmt(pair: Pair<Rule>) -> Result<Statement> {
    let mut inner = pair.into_inner();
    
    let target = parse_lvalue(inner.next()
        .ok_or_else(|| anyhow!("Missing assignment target"))?)?;
    
    let value = parse_expression(inner.next()
        .ok_or_else(|| anyhow!("Missing assignment value"))?)?;
    
    Ok(Statement::Assign { target, value })
}

fn parse_lvalue(pair: Pair<Rule>) -> Result<String> {
    // Simplified for now - just return the identifier
    let mut result = String::new();
    
    for item in pair.into_inner() {
        if item.as_rule() == Rule::identifier {
            if !result.is_empty() {
                result.push('.');
            }
            result.push_str(item.as_str());
        }
    }
    
    Ok(result)
}

fn parse_if_stmt(pair: Pair<Rule>) -> Result<Statement> {
    let mut inner = pair.into_inner();
    
    let condition = parse_expression(inner.next()
        .ok_or_else(|| anyhow!("Missing if condition"))?)?;
    
    let then_block = parse_block(inner.next()
        .ok_or_else(|| anyhow!("Missing then block"))?)?;
    
    let else_block = inner.next().map(|b| parse_block(b)).transpose()?;
    
    Ok(Statement::If {
        condition,
        then_block,
        else_block,
    })
}

fn parse_require_stmt(pair: Pair<Rule>) -> Result<Statement> {
    let mut inner = pair.into_inner();
    
    let condition = parse_expression(inner.next()
        .ok_or_else(|| anyhow!("Missing require condition"))?)?;
    
    let message = inner.next()
        .and_then(|p| {
            if p.as_rule() == Rule::string_lit {
                Some(parse_string_literal(p.as_str()))
            } else {
                None
            }
        });
    
    Ok(Statement::Require { condition, message })
}

fn parse_emit_stmt(pair: Pair<Rule>) -> Result<Statement> {
    let mut inner = pair.into_inner();
    
    let event = inner.next()
        .ok_or_else(|| anyhow!("Missing event name"))?
        .as_str()
        .to_string();
    
    let mut args = Vec::new();
    if let Some(arg_list) = inner.next() {
        if arg_list.as_rule() == Rule::arg_list {
            for arg in arg_list.into_inner() {
                args.push(parse_expression(arg)?);
            }
        }
    }
    
    Ok(Statement::Emit { event, args })
}

fn parse_return_stmt(pair: Pair<Rule>) -> Result<Statement> {
    let value = pair.into_inner()
        .next()
        .map(|p| parse_expression(p))
        .transpose()?;
    
    Ok(Statement::Return { value })
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expression> {
    match pair.as_rule() {
        Rule::expression | Rule::logical_or => parse_binary_expr(pair),
        Rule::primary => parse_primary(pair),
        _ => parse_binary_expr(pair),
    }
}

fn parse_binary_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or_else(|| anyhow!("Empty expression"))?;
    
    let mut left = if first.as_rule() == Rule::primary {
        parse_primary(first)?
    } else {
        parse_binary_expr(first)?
    };
    
    while let Some(op_or_right) = inner.next() {
        // This is simplified - in real implementation, handle operator precedence properly
        let op = parse_binary_op(op_or_right.as_str())?;
        let right = parse_expression(inner.next()
            .ok_or_else(|| anyhow!("Missing right operand"))?)?;
        
        left = Expression::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    
    Ok(left)
}

fn parse_primary(pair: Pair<Rule>) -> Result<Expression> {
    let inner = pair.into_inner().next()
        .unwrap_or(pair.clone());
    
    match inner.as_rule() {
        Rule::number_lit => {
            let num = inner.as_str().parse::<u64>()
                .context("Failed to parse number")?;
            Ok(Expression::Number(num))
        }
        Rule::bool_lit => {
            let b = inner.as_str() == "true";
            Ok(Expression::Bool(b))
        }
        Rule::string_lit => {
            Ok(Expression::String(parse_string_literal(inner.as_str())))
        }
        Rule::identifier => {
            // Check if it's a function call or just an identifier
            Ok(Expression::Identifier(inner.as_str().to_string()))
        }
        _ => Err(anyhow!("Unknown primary expression"))
    }
}

fn parse_binary_op(op_str: &str) -> Result<BinaryOp> {
    match op_str {
        "+" => Ok(BinaryOp::Add),
        "-" => Ok(BinaryOp::Sub),
        "*" => Ok(BinaryOp::Mul),
        "/" => Ok(BinaryOp::Div),
        "%" => Ok(BinaryOp::Mod),
        "==" => Ok(BinaryOp::Eq),
        "!=" => Ok(BinaryOp::Ne),
        "<" => Ok(BinaryOp::Lt),
        ">" => Ok(BinaryOp::Gt),
        "<=" => Ok(BinaryOp::Le),
        ">=" => Ok(BinaryOp::Ge),
        "&&" => Ok(BinaryOp::And),
        "||" => Ok(BinaryOp::Or),
        _ => Err(anyhow!("Unknown operator: {}", op_str))
    }
}

fn parse_string_literal(s: &str) -> String {
    // Remove quotes and handle escape sequences
    s.trim_matches('"')
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}