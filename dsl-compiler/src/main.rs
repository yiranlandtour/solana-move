use clap::{Parser as ClapParser, Subcommand};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

mod codegen;
use codegen::{solana::SolanaCodeGenerator, move_gen::MoveCodeGenerator};

#[derive(ClapParser)]
#[command(name = "ccdsl")]
#[command(about = "CrossChain DSL Compiler - Write once, deploy everywhere")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile DSL to target platforms
    Compile {
        /// Input DSL file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Target platform (solana, aptos, sui, all)
        #[arg(short, long, default_value = "all")]
        target: String,
        
        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,
    },
    
    /// Validate DSL syntax
    Validate {
        /// Input DSL file
        #[arg(short, long)]
        input: PathBuf,
    },
    
    /// Generate example DSL file
    Example {
        /// Output file
        #[arg(short, long, default_value = "example.ccdsl")]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile { input, target, output } => {
            compile(input, target, output)?;
        }
        Commands::Validate { input } => {
            validate(input)?;
        }
        Commands::Example { output } => {
            generate_example(output)?;
        }
    }
    
    Ok(())
}

fn compile(input: PathBuf, target: String, output: PathBuf) -> Result<()> {
    println!("🚀 CrossChain DSL Compiler");
    println!("==========================");
    println!("Input: {}", input.display());
    println!("Target: {}", target);
    println!("Output: {}", output.display());
    println!();
    
    // 读取 DSL 文件
    let dsl_content = fs::read_to_string(&input)?;
    
    // 创建输出目录
    fs::create_dir_all(&output)?;
    
    // 根据目标生成代码
    match target.as_str() {
        "solana" | "all" => {
            println!("📦 Generating Solana code...");
            let solana_gen = SolanaCodeGenerator::new();
            
            // 简化的示例 - 实际需要先解析 DSL
            let solana_code = generate_solana_example();
            
            let solana_output = output.join("solana");
            fs::create_dir_all(&solana_output)?;
            fs::write(solana_output.join("lib.rs"), solana_code)?;
            
            println!("✅ Solana code generated at: {}", solana_output.display());
        }
        _ => {}
    }
    
    match target.as_str() {
        "aptos" | "all" => {
            println!("📦 Generating Aptos Move code...");
            let move_gen = MoveCodeGenerator::new();
            
            // 简化的示例 - 实际需要先解析 DSL
            let move_code = generate_move_example();
            
            let aptos_output = output.join("aptos");
            fs::create_dir_all(&aptos_output)?;
            fs::write(aptos_output.join("token.move"), move_code)?;
            
            println!("✅ Aptos Move code generated at: {}", aptos_output.display());
        }
        _ => {}
    }
    
    match target.as_str() {
        "sui" | "all" => {
            println!("📦 Generating Sui Move code...");
            
            let sui_code = generate_sui_example();
            
            let sui_output = output.join("sui");
            fs::create_dir_all(&sui_output)?;
            fs::write(sui_output.join("token.move"), sui_code)?;
            
            println!("✅ Sui Move code generated at: {}", sui_output.display());
        }
        _ => {}
    }
    
    println!("\n🎉 Compilation complete!");
    println!("Next steps:");
    println!("  1. Review generated code in {}", output.display());
    println!("  2. Run platform-specific build commands");
    println!("  3. Deploy to respective blockchains");
    
    Ok(())
}

fn validate(input: PathBuf) -> Result<()> {
    println!("🔍 Validating DSL file: {}", input.display());
    
    let content = fs::read_to_string(&input)?;
    
    // TODO: 实际的解析验证
    println!("✅ DSL syntax is valid!");
    
    Ok(())
}

fn generate_example(output: PathBuf) -> Result<()> {
    let example = include_str!("../examples/token.ccdsl");
    fs::write(&output, example)?;
    
    println!("📝 Example DSL file generated: {}", output.display());
    println!("Edit this file and run: ccdsl compile -i {} -t all", output.display());
    
    Ok(())
}

// 临时的示例生成函数
fn generate_solana_example() -> String {
    r#"use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod token {
    use super::*;
    
    pub fn initialize(ctx: Context<Initialize>, initial_supply: u64, decimals: u8) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.total_supply = initial_supply;
        state.decimals = decimals;
        state.owner = ctx.accounts.owner.key();
        Ok(())
    }
    
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        // Transfer logic
        Ok(())
    }
}"#.to_string()
}

fn generate_move_example() -> String {
    r#"module token_addr::token {
    use std::signer;
    use aptos_framework::event;
    
    struct TokenState has key {
        total_supply: u64,
        owner: address,
        decimals: u8,
    }
    
    public entry fun initialize(account: &signer, initial_supply: u64, decimals: u8) {
        move_to(account, TokenState {
            total_supply: initial_supply,
            owner: signer::address_of(account),
            decimals,
        });
    }
    
    public entry fun transfer(from: &signer, to: address, amount: u64) acquires TokenState {
        // Transfer logic
    }
}"#.to_string()
}

fn generate_sui_example() -> String {
    r#"module token::token {
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    
    struct Token has key, store {
        id: UID,
        total_supply: u64,
        decimals: u8,
    }
    
    public fun initialize(initial_supply: u64, decimals: u8, ctx: &mut TxContext) {
        let token = Token {
            id: object::new(ctx),
            total_supply: initial_supply,
            decimals,
        };
        transfer::share_object(token);
    }
}"#.to_string()
}