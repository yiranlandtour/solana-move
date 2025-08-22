use super::super::{Contract, Type, Function, Statement, Expression, Visibility, BinaryOp, LValue};
use anyhow::Result;

pub struct MoveCodeGenerator;

impl MoveCodeGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, contract: &Contract) -> Result<String> {
        let move_code = self.transform_contract(contract);
        Ok(move_code)
    }

    fn transform_contract(&self, contract: &Contract) -> String {
        let mut code = String::new();
        
        // 模块声明
        code.push_str(&format!("module cross_chain::{} {{\n", contract.name.to_lowercase()));
        
        // 导入
        code.push_str("    use std::signer;\n");
        code.push_str("    use aptos_framework::event;\n");
        code.push_str("    use aptos_framework::timestamp;\n\n");
        
        // 生成资源结构
        if !contract.state.is_empty() {
            code.push_str("    /// Main state resource\n");
            code.push_str("    struct State has key {\n");
            for var in &contract.state {
                code.push_str("        ");
                code.push_str(&var.name);
                code.push_str(": ");
                code.push_str(&self.type_to_move(&var.ty));
                code.push_str(",\n");
            }
            code.push_str("    }\n\n");
        }
        
        // 生成函数
        for func in &contract.functions {
            code.push_str(&self.generate_function(func));
            code.push_str("\n");
        }
        
        code.push_str("}\n");
        code
    }

    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // 函数可见性
        match func.visibility {
            Visibility::Public => code.push_str("    public "),
            Visibility::Private => code.push_str("    "),
            Visibility::Internal => code.push_str("    public(friend) "),
            Visibility::External => code.push_str("    public "),
        }
        
        // entry 修饰符（如果是 public）
        if matches!(func.visibility, Visibility::Public) {
            code.push_str("entry ");
        }
        
        code.push_str("fun ");
        code.push_str(&func.name);
        code.push_str("(");
        
        // 第一个参数通常是 signer
        let mut first = true;
        if matches!(func.visibility, Visibility::Public) {
            code.push_str("account: &signer");
            first = false;
        }
        
        // 其他参数
        for param in &func.params {
            if !first {
                code.push_str(", ");
            }
            code.push_str(&param.name);
            code.push_str(": ");
            code.push_str(&self.type_to_move(&param.ty));
            first = false;
        }
        
        code.push_str(")");
        
        // 返回类型
        if let Some(ret_ty) = &func.return_type {
            code.push_str(": ");
            code.push_str(&self.type_to_move(ret_ty));
        }
        
        // acquires 声明
        code.push_str(" acquires State");
        
        code.push_str(" {\n");
        
        // 函数体
        for stmt in &func.body {
            code.push_str("        ");
            code.push_str(&self.statement_to_move(stmt));
            code.push_str("\n");
        }
        
        code.push_str("    }\n");
        code
    }

    fn type_to_move(&self, ty: &Type) -> String {
        match ty {
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::U128 => "u128".to_string(),
            Type::U256 => "u256".to_string(),
            Type::I8 => "u8".to_string(),  // Move doesn't have signed types
            Type::I16 => "u16".to_string(),
            Type::I32 => "u32".to_string(),
            Type::I64 => "u64".to_string(),
            Type::I128 => "u128".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Address => "address".to_string(),
            Type::String => "vector<u8>".to_string(),
            Type::Bytes => "vector<u8>".to_string(),
            Type::Map(k, v) => {
                // Move 使用 Table 或 SimpleMap
                format!("aptos_std::simple_map::SimpleMap<{}, {}>", 
                    self.type_to_move(k), self.type_to_move(v))
            },
            Type::Vec(t) => format!("vector<{}>", self.type_to_move(t)),
            Type::Array(t, _size) => format!("vector<{}>", self.type_to_move(t)),
            Type::Tuple(types) => {
                let types_str = types.iter()
                    .map(|t| self.type_to_move(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", types_str)
            },
            Type::Struct(name) => name.clone(),
            Type::Option(t) => format!("Option<{}>", self.type_to_move(t)),
            Type::Result(ok, err) => format!("Result<{}, {}>", self.type_to_move(ok), self.type_to_move(err)),
        }
    }

    fn statement_to_move(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Let { name, value, .. } => {
                format!("let {} = {};", name, self.expression_to_move(value))
            },
            Statement::Assign { target, value } => {
                // Move 中赋值需要处理可变引用
                let target_str = self.lvalue_to_move(target);
                format!("*{} = {};", target_str, self.expression_to_move(value))
            },
            Statement::Require { condition, message } => {
                if let Some(msg) = message {
                    format!("assert!({}, 1);", self.expression_to_move(condition))
                } else {
                    format!("assert!({}, 1);", self.expression_to_move(condition))
                }
            },
            Statement::If { condition, then_block, else_block } => {
                let mut code = format!("if ({}) {{\n", self.expression_to_move(condition));
                for s in then_block {
                    code.push_str("            ");
                    code.push_str(&self.statement_to_move(s));
                    code.push_str("\n");
                }
                code.push_str("        }");
                
                if let Some(else_b) = else_block {
                    code.push_str(" else {\n");
                    for s in else_b {
                        code.push_str("            ");
                        code.push_str(&self.statement_to_move(s));
                        code.push_str("\n");
                    }
                    code.push_str("        }");
                }
                code
            },
            Statement::Emit { event, args } => {
                format!("event::emit({} {{ /* fields */ }});", event)
            },
            Statement::Return { value } => {
                if let Some(v) = value {
                    self.expression_to_move(v)
                } else {
                    "".to_string()
                }
            },
            _ => "// TODO".to_string(),
        }
    }

    fn expression_to_move(&self, expr: &Expression) -> String {
        match expr {
            Expression::Number(n) => n.to_string(),
            Expression::Bool(b) => b.to_string(),
            Expression::String(s) => format!("b\"{}\"", s),
            Expression::Identifier(id) => id.clone(),
            Expression::Binary { op, left, right } => {
                format!("({} {} {})", 
                    self.expression_to_move(left),
                    self.binary_op_to_move(op),
                    self.expression_to_move(right))
            },
            Expression::Call { func, args } => {
                let func_str = self.expression_to_move(func);
                let args_str = args.iter()
                    .map(|a| self.expression_to_move(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", func_str, args_str)
            },
            _ => "/* expr */".to_string(),
        }
    }

    fn binary_op_to_move(&self, op: &BinaryOp) -> &str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => "^",  // Move doesn't have pow operator
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
        }
    }
    
    fn lvalue_to_move(&self, lvalue: &LValue) -> String {
        match lvalue {
            LValue::Identifier(name) => name.clone(),
            LValue::Index { array, index } => {
                // array is a String (identifier name)
                format!("{}[{}]", array, self.expression_to_move(index))
            },
        }
    }
}