use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow, Context};
use crate::{Contract, Function, Statement, Expression, Type, StateVariable, Parameter};

pub struct SemanticAnalyzer {
    // Symbol table for variables and their types
    symbols: HashMap<String, SymbolInfo>,
    // Current scope level
    scope_level: usize,
    // State variables from contract
    state_vars: HashMap<String, Type>,
    // Function signatures
    functions: HashMap<String, FunctionSignature>,
    // Track which variables are initialized
    initialized: HashSet<String>,
    // Errors collected during analysis
    errors: Vec<String>,
}

#[derive(Clone, Debug)]
struct SymbolInfo {
    ty: Type,
    scope_level: usize,
    is_mutable: bool,
    is_initialized: bool,
}

#[derive(Clone, Debug)]
struct FunctionSignature {
    params: Vec<(String, Type)>,
    return_type: Option<Type>,
    is_public: bool,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scope_level: 0,
            state_vars: HashMap::new(),
            functions: HashMap::new(),
            initialized: HashSet::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn analyze(&mut self, contract: &Contract) -> Result<()> {
        // First pass: Register state variables and function signatures
        self.register_state_vars(&contract.state)?;
        self.register_functions(&contract.functions)?;
        
        // Add built-in functions
        self.add_builtins();
        
        // Second pass: Analyze each function body
        for func in &contract.functions {
            self.analyze_function(func)?;
        }
        
        // Check for errors
        if !self.errors.is_empty() {
            return Err(anyhow!("Semantic errors:\n{}", self.errors.join("\n")));
        }
        
        Ok(())
    }
    
    fn register_state_vars(&mut self, vars: &[StateVariable]) -> Result<()> {
        for var in vars {
            if self.state_vars.contains_key(&var.name) {
                self.errors.push(format!("Duplicate state variable: {}", var.name));
            } else {
                self.state_vars.insert(var.name.clone(), var.ty.clone());
            }
        }
        Ok(())
    }
    
    fn register_functions(&mut self, functions: &[Function]) -> Result<()> {
        for func in functions {
            let sig = FunctionSignature {
                params: func.params.iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect(),
                return_type: func.return_type.clone(),
                is_public: matches!(func.visibility, crate::Visibility::Public),
            };
            
            if self.functions.contains_key(&func.name) {
                self.errors.push(format!("Duplicate function: {}", func.name));
            } else {
                self.functions.insert(func.name.clone(), sig);
            }
        }
        Ok(())
    }
    
    fn add_builtins(&mut self) {
        // Add built-in functions like msg_sender()
        self.functions.insert("msg_sender".to_string(), FunctionSignature {
            params: vec![],
            return_type: Some(Type::Address),
            is_public: false,
        });
    }
    
    fn analyze_function(&mut self, func: &Function) -> Result<()> {
        self.enter_scope();
        
        // Add parameters to symbol table
        for param in &func.params {
            self.add_symbol(&param.name, &param.ty, false)?;
        }
        
        // Analyze function body
        for stmt in &func.body {
            self.analyze_statement(stmt)?;
        }
        
        // Check return type consistency
        if let Some(expected_type) = &func.return_type {
            // TODO: Verify all return paths return the correct type
        }
        
        self.exit_scope();
        Ok(())
    }
    
    fn analyze_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, value } => {
                let value_type = self.infer_expression_type(value)?;
                self.add_symbol(name, &value_type, false)?;
                self.initialized.insert(name.clone());
            }
            
            Statement::Assign { target, value } => {
                // Check if target exists and is mutable
                let target_type = self.get_symbol_type(target)?;
                let value_type = self.infer_expression_type(value)?;
                
                if !self.types_compatible(&target_type, &value_type) {
                    self.errors.push(format!(
                        "Type mismatch: cannot assign {:?} to {:?}",
                        value_type, target_type
                    ));
                }
            }
            
            Statement::If { condition, then_block, else_block } => {
                let cond_type = self.infer_expression_type(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    self.errors.push("If condition must be boolean".to_string());
                }
                
                self.enter_scope();
                for s in then_block {
                    self.analyze_statement(s)?;
                }
                self.exit_scope();
                
                if let Some(else_stmts) = else_block {
                    self.enter_scope();
                    for s in else_stmts {
                        self.analyze_statement(s)?;
                    }
                    self.exit_scope();
                }
            }
            
            Statement::Require { condition, .. } => {
                let cond_type = self.infer_expression_type(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    self.errors.push("Require condition must be boolean".to_string());
                }
            }
            
            Statement::Emit { event, args } => {
                // Check event exists and arguments match
                for arg in args {
                    self.infer_expression_type(arg)?;
                }
            }
            
            Statement::Return { value } => {
                if let Some(expr) = value {
                    self.infer_expression_type(expr)?;
                }
            }
            
            Statement::Expression(expr) => {
                self.infer_expression_type(expr)?;
            }
        }
        
        Ok(())
    }
    
    fn infer_expression_type(&self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Number(_) => Ok(Type::U64),
            Expression::Bool(_) => Ok(Type::Bool),
            Expression::String(_) => Ok(Type::String),
            
            Expression::Identifier(name) => {
                self.get_symbol_type(name)
            }
            
            Expression::Binary { op, left, right } => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                
                use crate::BinaryOp::*;
                match op {
                    Add | Sub | Mul | Div | Mod => {
                        if matches!(left_type, Type::U8 | Type::U64 | Type::U128) &&
                           self.types_compatible(&left_type, &right_type) {
                            Ok(left_type)
                        } else {
                            Err(anyhow!("Arithmetic operations require numeric types"))
                        }
                    }
                    Eq | Ne => {
                        if self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(anyhow!("Cannot compare different types"))
                        }
                    }
                    Lt | Gt | Le | Ge => {
                        if matches!(left_type, Type::U8 | Type::U64 | Type::U128) &&
                           self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(anyhow!("Comparison requires numeric types"))
                        }
                    }
                    And | Or => {
                        if matches!(left_type, Type::Bool) && matches!(right_type, Type::Bool) {
                            Ok(Type::Bool)
                        } else {
                            Err(anyhow!("Logical operators require boolean types"))
                        }
                    }
                }
            }
            
            Expression::Unary { op, expr } => {
                let expr_type = self.infer_expression_type(expr)?;
                
                use crate::UnaryOp::*;
                match op {
                    Not => {
                        if matches!(expr_type, Type::Bool) {
                            Ok(Type::Bool)
                        } else {
                            Err(anyhow!("NOT operator requires boolean type"))
                        }
                    }
                    Neg => {
                        if matches!(expr_type, Type::U8 | Type::U64 | Type::U128) {
                            Ok(expr_type)
                        } else {
                            Err(anyhow!("Negation requires numeric type"))
                        }
                    }
                }
            }
            
            Expression::Call { func, args } => {
                if let Some(sig) = self.functions.get(func) {
                    // Check argument count
                    if args.len() != sig.params.len() {
                        return Err(anyhow!(
                            "Function {} expects {} arguments, got {}",
                            func, sig.params.len(), args.len()
                        ));
                    }
                    
                    // Check argument types
                    for (arg, (_, expected_type)) in args.iter().zip(&sig.params) {
                        let arg_type = self.infer_expression_type(arg)?;
                        if !self.types_compatible(&arg_type, expected_type) {
                            return Err(anyhow!(
                                "Type mismatch in function call {}",
                                func
                            ));
                        }
                    }
                    
                    sig.return_type.clone()
                        .ok_or_else(|| anyhow!("Function {} has no return type", func))
                } else {
                    Err(anyhow!("Unknown function: {}", func))
                }
            }
            
            Expression::Index { array, index } => {
                let array_type = self.get_symbol_type(array)?;
                let index_type = self.infer_expression_type(index)?;
                
                match array_type {
                    Type::Map(key_type, value_type) => {
                        if !self.types_compatible(&index_type, &key_type) {
                            return Err(anyhow!("Map index type mismatch"));
                        }
                        Ok(*value_type)
                    }
                    Type::Vec(elem_type) => {
                        if !matches!(index_type, Type::U64) {
                            return Err(anyhow!("Array index must be u64"));
                        }
                        Ok(*elem_type)
                    }
                    _ => Err(anyhow!("Cannot index non-collection type"))
                }
            }
            
            Expression::Field { object, field } => {
                // For now, just return a placeholder
                // In full implementation, would check struct fields
                Ok(Type::U64)
            }
        }
    }
    
    fn get_symbol_type(&self, name: &str) -> Result<Type> {
        // Check local symbols first
        if let Some(info) = self.symbols.get(name) {
            return Ok(info.ty.clone());
        }
        
        // Check state variables
        if let Some(ty) = self.state_vars.get(name) {
            return Ok(ty.clone());
        }
        
        Err(anyhow!("Unknown symbol: {}", name))
    }
    
    fn add_symbol(&mut self, name: &str, ty: &Type, is_mutable: bool) -> Result<()> {
        if self.symbols.contains_key(name) {
            self.errors.push(format!("Variable {} already declared", name));
            return Ok(());
        }
        
        self.symbols.insert(name.to_string(), SymbolInfo {
            ty: ty.clone(),
            scope_level: self.scope_level,
            is_mutable,
            is_initialized: false,
        });
        
        Ok(())
    }
    
    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::U8, Type::U8) | (Type::U64, Type::U64) | (Type::U128, Type::U128) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Address, Type::Address) => true,
            (Type::String, Type::String) => true,
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.types_compatible(k1, k2) && self.types_compatible(v1, v2)
            }
            (Type::Vec(e1), Type::Vec(e2)) => self.types_compatible(e1, e2),
            _ => false,
        }
    }
    
    fn enter_scope(&mut self) {
        self.scope_level += 1;
    }
    
    fn exit_scope(&mut self) {
        // Remove symbols from the exiting scope
        self.symbols.retain(|_, info| info.scope_level < self.scope_level);
        self.scope_level -= 1;
    }
}