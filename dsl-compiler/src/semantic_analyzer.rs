use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow, Context};
use crate::{
    Contract, Function, Statement, Expression, Type, StateVariable, 
    Parameter, Visibility, LValue, BinaryOp, UnaryOp, StructDefinition,
    EventDefinition, Modifier, Constant
};

/// Symbol information stored in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub kind: SymbolKind,
    pub mutable: bool,
    pub scope_level: usize,
    pub defined_at: Location,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    StateVariable,
    LocalVariable,
    Parameter,
    Function,
    Struct,
    Event,
    Modifier,
    Constant,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

/// Symbol table for tracking declarations and scopes
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    current_scope_level: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            current_scope_level: 0,
        }
    }
    
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope_level += 1;
    }
    
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            self.current_scope_level -= 1;
        }
    }
    
    pub fn declare(&mut self, symbol: Symbol) -> Result<()> {
        let current_scope = self.scopes.last_mut()
            .ok_or_else(|| anyhow!("No current scope"))?;
        
        if current_scope.contains_key(&symbol.name) {
            return Err(anyhow!("Symbol '{}' already declared in current scope", symbol.name));
        }
        
        current_scope.insert(symbol.name.clone(), symbol);
        Ok(())
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }
    
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                return scope.get_mut(name);
            }
        }
        None
    }
}

/// Type checking context
pub struct TypeContext {
    pub symbol_table: SymbolTable,
    pub current_function: Option<String>,
    pub current_return_type: Option<Type>,
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<SemanticWarning>,
    pub contract_name: String,
    pub structs: HashMap<String, StructDefinition>,
    pub events: HashMap<String, EventDefinition>,
    pub modifiers: HashMap<String, Modifier>,
}

#[derive(Debug)]
pub struct SemanticError {
    pub message: String,
    pub location: Option<Location>,
}

#[derive(Debug)]
pub struct SemanticWarning {
    pub message: String,
    pub location: Option<Location>,
}

impl TypeContext {
    pub fn new(contract_name: String) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            current_function: None,
            current_return_type: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            contract_name,
            structs: HashMap::new(),
            events: HashMap::new(),
            modifiers: HashMap::new(),
        }
    }
    
    pub fn add_error(&mut self, message: String) {
        self.errors.push(SemanticError {
            message,
            location: None,
        });
    }
    
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(SemanticWarning {
            message,
            location: None,
        });
    }
}

/// Type inference engine
pub struct TypeInference {
    constraints: Vec<TypeConstraint>,
    substitutions: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
enum TypeConstraint {
    Equal(Type, Type),
    Subtype(Type, Type),
}

impl TypeInference {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            substitutions: HashMap::new(),
        }
    }
    
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }
    
    pub fn solve(&mut self) -> Result<()> {
        // Simple unification algorithm
        while !self.constraints.is_empty() {
            let constraint = self.constraints.pop().unwrap();
            match constraint {
                TypeConstraint::Equal(t1, t2) => {
                    self.unify(t1, t2)?;
                }
                TypeConstraint::Subtype(sub, super_) => {
                    // Check if sub is a subtype of super
                    if !self.is_subtype(&sub, &super_) {
                        return Err(anyhow!("{:?} is not a subtype of {:?}", sub, super_));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn unify(&mut self, t1: Type, t2: Type) -> Result<()> {
        let t1_str = format!("{:?}", t1);
        let t2_str = format!("{:?}", t2);
        
        match (t1, t2) {
            (Type::U8, Type::U8) | (Type::U64, Type::U64) | 
            (Type::Bool, Type::Bool) | (Type::Address, Type::Address) => Ok(()),
            
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.unify(*k1, *k2)?;
                self.unify(*v1, *v2)
            }
            
            (Type::Vec(t1), Type::Vec(t2)) => self.unify(*t1, *t2),
            
            (Type::Option(t1), Type::Option(t2)) => self.unify(*t1, *t2),
            
            (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
                self.unify(*ok1, *ok2)?;
                self.unify(*err1, *err2)
            }
            
            _ => Err(anyhow!("Cannot unify {} with {}", t1_str, t2_str))
        }
    }
    
    fn is_subtype(&self, sub: &Type, super_: &Type) -> bool {
        match (sub, super_) {
            // Numeric promotions
            (Type::U8, Type::U16) | (Type::U8, Type::U32) | 
            (Type::U8, Type::U64) | (Type::U8, Type::U128) => true,
            (Type::U16, Type::U32) | (Type::U16, Type::U64) | 
            (Type::U16, Type::U128) => true,
            (Type::U32, Type::U64) | (Type::U32, Type::U128) => true,
            (Type::U64, Type::U128) => true,
            
            // Same types are subtypes of themselves
            (t1, t2) if t1 == t2 => true,
            
            _ => false
        }
    }
}

/// Main semantic analyzer
pub struct SemanticAnalyzer {
    context: TypeContext,
    type_inference: TypeInference,
}

impl SemanticAnalyzer {
    pub fn new(contract_name: String) -> Self {
        Self {
            context: TypeContext::new(contract_name),
            type_inference: TypeInference::new(),
        }
    }
    
    pub fn analyze(&mut self, contract: &Contract) -> Result<()> {
        // First pass: Register all type definitions
        self.register_types(contract)?;
        
        // Second pass: Register state variables
        self.register_state_variables(contract)?;
        
        // Third pass: Check all functions
        for function in &contract.functions {
            self.check_function(function)?;
        }
        
        // Fourth pass: Solve type constraints
        self.type_inference.solve()?;
        
        // Check for errors
        if !self.context.errors.is_empty() {
            let error_messages: Vec<String> = self.context.errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            return Err(anyhow!("Semantic errors:\n{}", error_messages.join("\n")));
        }
        
        Ok(())
    }
    
    fn register_types(&mut self, contract: &Contract) -> Result<()> {
        // Register structs
        for struct_def in &contract.structs {
            self.context.structs.insert(struct_def.name.clone(), struct_def.clone());
        }
        
        // Register events
        for event in &contract.events {
            self.context.events.insert(event.name.clone(), event.clone());
        }
        
        // Register modifiers
        for modifier in &contract.modifiers {
            self.context.modifiers.insert(modifier.name.clone(), modifier.clone());
        }
        
        Ok(())
    }
    
    fn register_state_variables(&mut self, contract: &Contract) -> Result<()> {
        for state_var in &contract.state {
            let symbol = Symbol {
                name: state_var.name.clone(),
                ty: state_var.ty.clone(),
                kind: SymbolKind::StateVariable,
                mutable: state_var.is_mutable,
                scope_level: 0,
                defined_at: Location { line: 0, column: 0 },
            };
            
            self.context.symbol_table.declare(symbol)?;
        }
        
        Ok(())
    }
    
    fn check_function(&mut self, function: &Function) -> Result<()> {
        // Set current function context
        self.context.current_function = Some(function.name.clone());
        self.context.current_return_type = function.return_type.clone();
        
        // Enter new scope for function
        self.context.symbol_table.enter_scope();
        
        // Register parameters
        for param in &function.params {
            let symbol = Symbol {
                name: param.name.clone(),
                ty: param.ty.clone(),
                kind: SymbolKind::Parameter,
                mutable: param.is_mutable,
                scope_level: self.context.symbol_table.current_scope_level,
                defined_at: Location { line: 0, column: 0 },
            };
            
            self.context.symbol_table.declare(symbol)?;
        }
        
        // Check function body
        for statement in &function.body {
            self.check_statement(statement)?;
        }
        
        // Check return type consistency
        if let Some(expected_return) = &function.return_type {
            // Verify that all code paths return the correct type
            if !self.all_paths_return(&function.body) {
                self.context.add_error(format!(
                    "Function '{}' must return a value of type {:?} on all paths",
                    function.name, expected_return
                ));
            }
        }
        
        // Exit function scope
        self.context.symbol_table.exit_scope();
        
        Ok(())
    }
    
    fn check_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Let { name, ty, value, is_mutable } => {
                // Infer or check type
                let value_type = self.infer_expression_type(value)?;
                
                let actual_type = if let Some(declared_type) = ty {
                    // Check that value type matches declared type
                    self.type_inference.add_constraint(
                        TypeConstraint::Equal(value_type.clone(), declared_type.clone())
                    );
                    declared_type.clone()
                } else {
                    value_type
                };
                
                // Declare variable
                let symbol = Symbol {
                    name: name.clone(),
                    ty: actual_type,
                    kind: SymbolKind::LocalVariable,
                    mutable: *is_mutable,
                    scope_level: self.context.symbol_table.current_scope_level,
                    defined_at: Location { line: 0, column: 0 },
                };
                
                self.context.symbol_table.declare(symbol)?;
            }
            
            Statement::Assign { target, value } => {
                // Check that target exists and get its type
                let target_type = self.check_lvalue(target)?;
                let value_type = self.infer_expression_type(value)?;
                
                // Check mutability separately
                self.check_lvalue_mutability(target);
                
                // Types must match
                self.type_inference.add_constraint(
                    TypeConstraint::Equal(target_type, value_type)
                );
            }
            
            Statement::If { condition, then_block, else_block } => {
                // Condition must be boolean
                let cond_type = self.infer_expression_type(condition)?;
                self.type_inference.add_constraint(
                    TypeConstraint::Equal(cond_type, Type::Bool)
                );
                
                // Check then block
                self.context.symbol_table.enter_scope();
                for stmt in then_block {
                    self.check_statement(stmt)?;
                }
                self.context.symbol_table.exit_scope();
                
                // Check else block if present
                if let Some(else_stmts) = else_block {
                    self.context.symbol_table.enter_scope();
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                    self.context.symbol_table.exit_scope();
                }
            }
            
            Statement::While { condition, body } => {
                // Condition must be boolean
                let cond_type = self.infer_expression_type(condition)?;
                self.type_inference.add_constraint(
                    TypeConstraint::Equal(cond_type, Type::Bool)
                );
                
                // Check body
                self.context.symbol_table.enter_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.context.symbol_table.exit_scope();
            }
            
            Statement::Return { value } => {
                if let Some(expr) = value {
                    let return_type = self.infer_expression_type(expr)?;
                    
                    if let Some(expected) = &self.context.current_return_type {
                        self.type_inference.add_constraint(
                            TypeConstraint::Equal(return_type, expected.clone())
                        );
                    } else {
                        self.context.add_error(
                            "Return statement in void function".to_string()
                        );
                    }
                } else {
                    if self.context.current_return_type.is_some() {
                        self.context.add_error(
                            "Return statement must return a value".to_string()
                        );
                    }
                }
            }
            
            Statement::Require { condition, message: _ } |
            Statement::Assert { condition, message: _ } => {
                // Condition must be boolean
                let cond_type = self.infer_expression_type(condition)?;
                self.type_inference.add_constraint(
                    TypeConstraint::Equal(cond_type, Type::Bool)
                );
            }
            
            Statement::Emit { event, args } => {
                // Check that event exists
                if !self.context.events.contains_key(event) {
                    self.context.add_error(format!("Unknown event '{}'", event));
                } else {
                    // Check arguments
                    for arg in args {
                        self.infer_expression_type(arg)?;
                    }
                }
            }
            
            Statement::Expression(expr) => {
                self.infer_expression_type(expr)?;
            }
            
            _ => {} // Handle other statement types
        }
        
        Ok(())
    }
    
    fn check_lvalue_mutability(&mut self, lvalue: &LValue) {
        match lvalue {
            LValue::Identifier(name) => {
                if let Some(symbol) = self.context.symbol_table.lookup(name) {
                    if !symbol.mutable {
                        self.context.add_error(format!(
                            "Cannot assign to immutable variable '{}'", name
                        ));
                    }
                }
            }
            LValue::Index { array, .. } => {
                self.check_lvalue_mutability(array);
            }
            LValue::Field { object, .. } => {
                self.check_lvalue_mutability(object);
            }
        }
    }
    
    fn check_lvalue(&mut self, lvalue: &LValue) -> Result<Type> {
        match lvalue {
            LValue::Identifier(name) => {
                if let Some(symbol) = self.context.symbol_table.lookup(name) {
                    Ok(symbol.ty.clone())
                } else {
                    Err(anyhow!("Undefined variable '{}'", name))
                }
            }
            
            LValue::Index { array, index } => {
                let array_type = self.check_lvalue(array)?;
                let index_type = self.infer_expression_type(index)?;
                
                match array_type {
                    Type::Map(_, value_type) => {
                        // Map indexing
                        Ok(*value_type)
                    }
                    Type::Vec(elem_type) | Type::Array(elem_type, _) => {
                        // Array/Vec indexing - index must be numeric
                        self.type_inference.add_constraint(
                            TypeConstraint::Subtype(index_type, Type::U64)
                        );
                        Ok(*elem_type)
                    }
                    _ => Err(anyhow!("Cannot index type {:?}", array_type))
                }
            }
            
            LValue::Field { object, field } => {
                let object_type = self.check_lvalue(object)?;
                
                match object_type {
                    Type::Struct(struct_name) => {
                        if let Some(struct_def) = self.context.structs.get(&struct_name) {
                            for field_def in &struct_def.fields {
                                if field_def.name == *field {
                                    return Ok(field_def.ty.clone());
                                }
                            }
                            Err(anyhow!("Struct '{}' has no field '{}'", struct_name, field))
                        } else {
                            Err(anyhow!("Unknown struct type '{}'", struct_name))
                        }
                    }
                    _ => Err(anyhow!("Cannot access field on type {:?}", object_type))
                }
            }
        }
    }
    
    fn infer_expression_type(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Number(_) => Ok(Type::U64), // Default to U64
            Expression::Float(_) => Ok(Type::U64), // No float type, convert to U64
            Expression::Bool(_) => Ok(Type::Bool),
            Expression::String(_) => Ok(Type::String),
            Expression::Bytes(_) => Ok(Type::Bytes),
            
            Expression::Identifier(name) => {
                if let Some(symbol) = self.context.symbol_table.lookup(name) {
                    Ok(symbol.ty.clone())
                } else {
                    Err(anyhow!("Undefined identifier '{}'", name))
                }
            }
            
            Expression::Binary { op, left, right } => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | 
                    BinaryOp::Div | BinaryOp::Mod => {
                        // Numeric operations
                        self.type_inference.add_constraint(
                            TypeConstraint::Equal(left_type.clone(), right_type)
                        );
                        Ok(left_type)
                    }
                    
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | 
                    BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        // Comparison operations
                        self.type_inference.add_constraint(
                            TypeConstraint::Equal(left_type, right_type)
                        );
                        Ok(Type::Bool)
                    }
                    
                    BinaryOp::And | BinaryOp::Or => {
                        // Logical operations
                        self.type_inference.add_constraint(
                            TypeConstraint::Equal(left_type, Type::Bool)
                        );
                        self.type_inference.add_constraint(
                            TypeConstraint::Equal(right_type, Type::Bool)
                        );
                        Ok(Type::Bool)
                    }
                    
                    _ => Ok(left_type) // For other operators
                }
            }
            
            Expression::Unary { op, expr } => {
                let expr_type = self.infer_expression_type(expr)?;
                
                match op {
                    UnaryOp::Not => {
                        self.type_inference.add_constraint(
                            TypeConstraint::Equal(expr_type, Type::Bool)
                        );
                        Ok(Type::Bool)
                    }
                    UnaryOp::Neg => {
                        // Must be numeric
                        Ok(expr_type)
                    }
                    _ => Ok(expr_type)
                }
            }
            
            Expression::Call { func, args: _ } => {
                // Look up function type
                if let Expression::Identifier(_func_name) = &**func {
                    // Find function in contract
                    // This is simplified - would need to look up actual function
                    Ok(Type::U64) // Placeholder
                } else {
                    Err(anyhow!("Complex function calls not yet supported"))
                }
            }
            
            Expression::MsgSender => Ok(Type::Address),
            Expression::MsgValue => Ok(Type::U64),
            Expression::BlockNumber => Ok(Type::U64),
            Expression::BlockTimestamp => Ok(Type::U64),
            
            _ => Ok(Type::U64) // Default for unhandled cases
        }
    }
    
    fn all_paths_return(&self, statements: &[Statement]) -> bool {
        // Simplified check - would need more sophisticated control flow analysis
        for stmt in statements {
            if matches!(stmt, Statement::Return { .. }) {
                return true;
            }
            if let Statement::If { then_block, else_block, .. } = stmt {
                if let Some(else_stmts) = else_block {
                    if self.all_paths_return(then_block) && self.all_paths_return(else_stmts) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.context.errors
    }
    
    pub fn get_warnings(&self) -> &[SemanticWarning] {
        &self.context.warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        
        let symbol = Symbol {
            name: "test".to_string(),
            ty: Type::U64,
            kind: SymbolKind::LocalVariable,
            mutable: true,
            scope_level: 0,
            defined_at: Location { line: 1, column: 1 },
        };
        
        assert!(table.declare(symbol.clone()).is_ok());
        assert!(table.lookup("test").is_some());
        
        table.enter_scope();
        assert!(table.lookup("test").is_some()); // Still visible
        
        table.exit_scope();
        assert!(table.lookup("test").is_some());
    }
    
    #[test]
    fn test_type_inference() {
        let mut inference = TypeInference::new();
        
        inference.add_constraint(TypeConstraint::Equal(Type::U64, Type::U64));
        assert!(inference.solve().is_ok());
        
        let mut inference2 = TypeInference::new();
        inference2.add_constraint(TypeConstraint::Equal(Type::U64, Type::Bool));
        assert!(inference2.solve().is_err());
    }
}