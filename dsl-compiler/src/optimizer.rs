use crate::{Contract, Function, Statement, Expression, BinaryOp, Type};
use std::collections::HashMap;

pub struct Optimizer {
    constant_values: HashMap<String, Expression>,
    dead_code_removed: usize,
    constants_folded: usize,
    expressions_simplified: usize,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            constant_values: HashMap::new(),
            dead_code_removed: 0,
            constants_folded: 0,
            expressions_simplified: 0,
        }
    }
    
    pub fn optimize(&mut self, contract: &mut Contract) {
        println!("🔧 Starting optimization pass...");
        
        // Optimize each function
        for func in &mut contract.functions {
            self.optimize_function(func);
        }
        
        // Report optimization statistics
        self.report_stats();
    }
    
    fn optimize_function(&mut self, func: &mut Function) {
        // Clear constant tracking for new function
        self.constant_values.clear();
        
        // Optimize function body
        func.body = self.optimize_statements(func.body.clone());
    }
    
    fn optimize_statements(&mut self, statements: Vec<Statement>) -> Vec<Statement> {
        let mut optimized = Vec::new();
        
        for stmt in statements {
            match self.optimize_statement(stmt) {
                Some(s) => optimized.push(s),
                None => self.dead_code_removed += 1,
            }
        }
        
        optimized
    }
    
    fn optimize_statement(&mut self, stmt: Statement) -> Option<Statement> {
        match stmt {
            Statement::Let { name, value } => {
                let optimized_value = self.optimize_expression(value);
                
                // Track constant values for propagation
                if self.is_constant(&optimized_value) {
                    self.constant_values.insert(name.clone(), optimized_value.clone());
                }
                
                Some(Statement::Let { name, value: optimized_value })
            }
            
            Statement::Assign { target, value } => {
                let optimized_value = self.optimize_expression(value);
                
                // Update constant tracking
                if self.is_constant(&optimized_value) {
                    self.constant_values.insert(target.clone(), optimized_value.clone());
                } else {
                    self.constant_values.remove(&target);
                }
                
                Some(Statement::Assign { target, value: optimized_value })
            }
            
            Statement::If { condition, then_block, else_block } => {
                let optimized_condition = self.optimize_expression(condition);
                
                // Check for constant conditions
                match &optimized_condition {
                    Expression::Bool(true) => {
                        // Always true - remove else block
                        self.dead_code_removed += else_block.as_ref().map(|b| b.len()).unwrap_or(0);
                        return Some(Statement::Expression(Expression::Bool(true)));
                    }
                    Expression::Bool(false) => {
                        // Always false - use else block or remove
                        self.dead_code_removed += then_block.len();
                        if let Some(else_stmts) = else_block {
                            return Some(Statement::Expression(Expression::Bool(false)));
                        }
                        return None;
                    }
                    _ => {}
                }
                
                Some(Statement::If {
                    condition: optimized_condition,
                    then_block: self.optimize_statements(then_block),
                    else_block: else_block.map(|b| self.optimize_statements(b)),
                })
            }
            
            Statement::Require { condition, message } => {
                let optimized_condition = self.optimize_expression(condition);
                
                // Check for always-true requires (can be removed)
                if let Expression::Bool(true) = optimized_condition {
                    self.dead_code_removed += 1;
                    return None;
                }
                
                Some(Statement::Require {
                    condition: optimized_condition,
                    message,
                })
            }
            
            Statement::Return { value } => {
                Some(Statement::Return {
                    value: value.map(|v| self.optimize_expression(v)),
                })
            }
            
            Statement::Emit { event, args } => {
                Some(Statement::Emit {
                    event,
                    args: args.into_iter()
                        .map(|a| self.optimize_expression(a))
                        .collect(),
                })
            }
            
            Statement::Expression(expr) => {
                let optimized = self.optimize_expression(expr);
                
                // Remove no-op expressions
                if self.is_no_op(&optimized) {
                    self.dead_code_removed += 1;
                    None
                } else {
                    Some(Statement::Expression(optimized))
                }
            }
        }
    }
    
    fn optimize_expression(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::Identifier(name) => {
                // Constant propagation
                if let Some(const_value) = self.constant_values.get(&name) {
                    self.expressions_simplified += 1;
                    return const_value.clone();
                }
                Expression::Identifier(name)
            }
            
            Expression::Binary { op, left, right } => {
                let left_opt = self.optimize_expression(*left);
                let right_opt = self.optimize_expression(*right);
                
                // Constant folding
                if let (Expression::Number(l), Expression::Number(r)) = (&left_opt, &right_opt) {
                    self.constants_folded += 1;
                    return self.fold_binary_op(op, *l, *r);
                }
                
                // Algebraic simplifications
                match (&op, &left_opt, &right_opt) {
                    // x + 0 = x, 0 + x = x
                    (BinaryOp::Add, expr, Expression::Number(0)) |
                    (BinaryOp::Add, Expression::Number(0), expr) => {
                        self.expressions_simplified += 1;
                        return expr.clone();
                    }
                    
                    // x - 0 = x
                    (BinaryOp::Sub, expr, Expression::Number(0)) => {
                        self.expressions_simplified += 1;
                        return expr.clone();
                    }
                    
                    // x * 1 = x, 1 * x = x
                    (BinaryOp::Mul, expr, Expression::Number(1)) |
                    (BinaryOp::Mul, Expression::Number(1), expr) => {
                        self.expressions_simplified += 1;
                        return expr.clone();
                    }
                    
                    // x * 0 = 0, 0 * x = 0
                    (BinaryOp::Mul, _, Expression::Number(0)) |
                    (BinaryOp::Mul, Expression::Number(0), _) => {
                        self.expressions_simplified += 1;
                        return Expression::Number(0);
                    }
                    
                    // x / 1 = x
                    (BinaryOp::Div, expr, Expression::Number(1)) => {
                        self.expressions_simplified += 1;
                        return expr.clone();
                    }
                    
                    // x && true = x, true && x = x
                    (BinaryOp::And, expr, Expression::Bool(true)) |
                    (BinaryOp::And, Expression::Bool(true), expr) => {
                        self.expressions_simplified += 1;
                        return expr.clone();
                    }
                    
                    // x && false = false, false && x = false
                    (BinaryOp::And, _, Expression::Bool(false)) |
                    (BinaryOp::And, Expression::Bool(false), _) => {
                        self.expressions_simplified += 1;
                        return Expression::Bool(false);
                    }
                    
                    // x || false = x, false || x = x
                    (BinaryOp::Or, expr, Expression::Bool(false)) |
                    (BinaryOp::Or, Expression::Bool(false), expr) => {
                        self.expressions_simplified += 1;
                        return expr.clone();
                    }
                    
                    // x || true = true, true || x = true
                    (BinaryOp::Or, _, Expression::Bool(true)) |
                    (BinaryOp::Or, Expression::Bool(true), _) => {
                        self.expressions_simplified += 1;
                        return Expression::Bool(true);
                    }
                    
                    _ => {}
                }
                
                Expression::Binary {
                    op,
                    left: Box::new(left_opt),
                    right: Box::new(right_opt),
                }
            }
            
            Expression::Unary { op, expr } => {
                let expr_opt = self.optimize_expression(*expr);
                
                // Constant folding for unary operations
                match (&op, &expr_opt) {
                    (crate::UnaryOp::Not, Expression::Bool(b)) => {
                        self.constants_folded += 1;
                        Expression::Bool(!b)
                    }
                    (crate::UnaryOp::Neg, Expression::Number(n)) => {
                        self.constants_folded += 1;
                        // Note: This would need proper handling for unsigned types
                        Expression::Number((!n).wrapping_add(1))
                    }
                    _ => Expression::Unary {
                        op,
                        expr: Box::new(expr_opt),
                    }
                }
            }
            
            Expression::Call { func, args } => {
                Expression::Call {
                    func,
                    args: args.into_iter()
                        .map(|a| self.optimize_expression(a))
                        .collect(),
                }
            }
            
            Expression::Index { array, index } => {
                Expression::Index {
                    array,
                    index: Box::new(self.optimize_expression(*index)),
                }
            }
            
            _ => expr,
        }
    }
    
    fn fold_binary_op(&self, op: BinaryOp, left: u64, right: u64) -> Expression {
        match op {
            BinaryOp::Add => Expression::Number(left.wrapping_add(right)),
            BinaryOp::Sub => Expression::Number(left.wrapping_sub(right)),
            BinaryOp::Mul => Expression::Number(left.wrapping_mul(right)),
            BinaryOp::Div => {
                if right != 0 {
                    Expression::Number(left / right)
                } else {
                    // Keep division by zero as-is for error reporting
                    Expression::Binary {
                        op: BinaryOp::Div,
                        left: Box::new(Expression::Number(left)),
                        right: Box::new(Expression::Number(0)),
                    }
                }
            }
            BinaryOp::Mod => {
                if right != 0 {
                    Expression::Number(left % right)
                } else {
                    Expression::Binary {
                        op: BinaryOp::Mod,
                        left: Box::new(Expression::Number(left)),
                        right: Box::new(Expression::Number(0)),
                    }
                }
            }
            BinaryOp::Eq => Expression::Bool(left == right),
            BinaryOp::Ne => Expression::Bool(left != right),
            BinaryOp::Lt => Expression::Bool(left < right),
            BinaryOp::Gt => Expression::Bool(left > right),
            BinaryOp::Le => Expression::Bool(left <= right),
            BinaryOp::Ge => Expression::Bool(left >= right),
            _ => {
                // Can't fold logical operators on numbers
                Expression::Binary {
                    op,
                    left: Box::new(Expression::Number(left)),
                    right: Box::new(Expression::Number(right)),
                }
            }
        }
    }
    
    fn is_constant(&self, expr: &Expression) -> bool {
        matches!(expr, Expression::Number(_) | Expression::Bool(_) | Expression::String(_))
    }
    
    fn is_no_op(&self, expr: &Expression) -> bool {
        // Identify expressions that have no effect
        matches!(expr, Expression::Number(_) | Expression::Bool(_) | Expression::String(_))
    }
    
    fn report_stats(&self) {
        println!("✨ Optimization complete:");
        println!("   - Dead code removed: {} statements", self.dead_code_removed);
        println!("   - Constants folded: {} expressions", self.constants_folded);
        println!("   - Expressions simplified: {} operations", self.expressions_simplified);
    }
}