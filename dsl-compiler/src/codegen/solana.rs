use super::super::{Contract, Type, Function, Statement, Expression, Visibility, BinaryOp};
use anyhow::Result;
use handlebars::Handlebars;

pub struct SolanaCodeGenerator {
    handlebars: Handlebars<'static>,
}

impl SolanaCodeGenerator {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("program", SOLANA_TEMPLATE).unwrap();
        Self { handlebars }
    }

    pub fn generate(&self, contract: &Contract) -> Result<String> {
        let solana_code = self.transform_contract(contract);
        Ok(solana_code)
    }

    fn transform_contract(&self, contract: &Contract) -> String {
        let mut code = String::new();
        
        // 生成 Anchor 程序头
        code.push_str("use anchor_lang::prelude::*;\n\n");
        code.push_str("declare_id!(\"11111111111111111111111111111111\");\n\n");
        
        // 生成程序模块
        code.push_str(&format!("#[program]\npub mod {} {{\n", contract.name.to_lowercase()));
        code.push_str("    use super::*;\n\n");
        
        // 生成函数
        for func in &contract.functions {
            code.push_str(&self.generate_function(func));
            code.push_str("\n");
        }
        
        code.push_str("}\n\n");
        
        // 生成账户结构
        code.push_str(&self.generate_accounts(&contract));
        
        // 生成状态结构
        code.push_str(&self.generate_state(&contract));
        
        // 生成错误码
        code.push_str(&self.generate_errors());
        
        code
    }

    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // 函数签名
        code.push_str("    pub fn ");
        code.push_str(&func.name);
        code.push_str("(ctx: Context<");
        code.push_str(&capitalize(&func.name));
        code.push_str(">");
        
        // 添加参数
        for param in &func.params {
            code.push_str(", ");
            code.push_str(&param.name);
            code.push_str(": ");
            code.push_str(&self.type_to_rust(&param.ty));
        }
        
        code.push_str(") -> Result<()> {\n");
        
        // 函数体
        for stmt in &func.body {
            code.push_str("        ");
            code.push_str(&self.statement_to_rust(stmt));
            code.push_str("\n");
        }
        
        code.push_str("        Ok(())\n");
        code.push_str("    }\n");
        
        code
    }

    fn generate_accounts(&self, contract: &Contract) -> String {
        let mut code = String::new();
        
        // 为每个函数生成 Accounts 结构
        for func in &contract.functions {
            code.push_str("#[derive(Accounts)]\n");
            code.push_str(&format!("pub struct {} {{\n", capitalize(&func.name)));
            
            // 基本账户
            code.push_str("    #[account(mut)]\n");
            code.push_str("    pub user: Signer<'info>,\n");
            
            // 状态账户
            if !contract.state.is_empty() {
                code.push_str("    #[account(\n");
                code.push_str("        mut,\n");
                code.push_str("        seeds = [b\"state\"],\n");
                code.push_str("        bump\n");
                code.push_str("    )]\n");
                code.push_str("    pub state: Account<'info, State>,\n");
            }
            
            code.push_str("    pub system_program: Program<'info, System>,\n");
            code.push_str("}\n\n");
        }
        
        code
    }

    fn generate_state(&self, contract: &Contract) -> String {
        let mut code = String::new();
        
        code.push_str("#[account]\n");
        code.push_str("pub struct State {\n");
        
        for var in &contract.state {
            code.push_str("    pub ");
            code.push_str(&var.name);
            code.push_str(": ");
            code.push_str(&self.type_to_rust(&var.ty));
            code.push_str(",\n");
        }
        
        code.push_str("}\n\n");
        code
    }

    fn generate_errors(&self) -> String {
        let mut code = String::new();
        
        code.push_str("#[error_code]\n");
        code.push_str("pub enum ErrorCode {\n");
        code.push_str("    #[msg(\"Unauthorized\")]\n");
        code.push_str("    Unauthorized,\n");
        code.push_str("    #[msg(\"Insufficient balance\")]\n");
        code.push_str("    InsufficientBalance,\n");
        code.push_str("    #[msg(\"Invalid parameter\")]\n");
        code.push_str("    InvalidParameter,\n");
        code.push_str("}\n");
        
        code
    }

    fn type_to_rust(&self, ty: &Type) -> String {
        match ty {
            Type::U8 => "u8".to_string(),
            Type::U64 => "u64".to_string(),
            Type::U128 => "u128".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Address => "Pubkey".to_string(),
            Type::String => "String".to_string(),
            Type::Map(k, v) => format!("HashMap<{}, {}>", 
                self.type_to_rust(k), self.type_to_rust(v)),
            Type::Vec(t) => format!("Vec<{}>", self.type_to_rust(t)),
        }
    }

    fn statement_to_rust(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Let { name, value } => {
                format!("let {} = {};", name, self.expression_to_rust(value))
            },
            Statement::Assign { target, value } => {
                format!("{} = {};", target, self.expression_to_rust(value))
            },
            Statement::Require { condition, message } => {
                if let Some(msg) = message {
                    format!("require!({}, ErrorCode::InvalidParameter);", 
                        self.expression_to_rust(condition))
                } else {
                    format!("require!({});", self.expression_to_rust(condition))
                }
            },
            Statement::Emit { event, args } => {
                format!("emit!({} {{ /* fields */ }});", event)
            },
            Statement::Return { value } => {
                if let Some(v) = value {
                    format!("return Ok({});", self.expression_to_rust(v))
                } else {
                    "return Ok(());".to_string()
                }
            },
            _ => "// TODO".to_string(),
        }
    }

    fn expression_to_rust(&self, expr: &Expression) -> String {
        match expr {
            Expression::Number(n) => n.to_string(),
            Expression::Bool(b) => b.to_string(),
            Expression::String(s) => format!("\"{}\"", s),
            Expression::Identifier(id) => id.clone(),
            Expression::Binary { op, left, right } => {
                format!("({} {} {})", 
                    self.expression_to_rust(left),
                    self.binary_op_to_rust(op),
                    self.expression_to_rust(right))
            },
            _ => "/* expr */".to_string(),
        }
    }

    fn binary_op_to_rust(&self, op: &BinaryOp) -> &str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

const SOLANA_TEMPLATE: &str = r#"
use anchor_lang::prelude::*;

declare_id!("{{program_id}}");

#[program]
pub mod {{contract_name}} {
    use super::*;
    
    {{#each functions}}
    pub fn {{this.name}}(ctx: Context<{{capitalize this.name}}>) -> Result<()> {
        {{#each this.body}}
        {{this}}
        {{/each}}
        Ok(())
    }
    {{/each}}
}
"#;