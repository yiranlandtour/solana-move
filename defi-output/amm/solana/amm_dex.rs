use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use std::collections::HashMap;

declare_id!("AMM11111111111111111111111111111111111111111");

#[program]
pub mod amm_dex {
    use super::*;
    
    pub fn initialize(
        ctx: Context<Initialize>,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<()> {
        let dex = &mut ctx.accounts.dex_state;
        dex.admin = ctx.accounts.admin.key();
        dex.fee_numerator = fee_numerator;
        dex.fee_denominator = fee_denominator;
        dex.protocol_fee_share = 6;
        dex.pool_count = 0;
        dex.paused = false;
        
        emit!(DexInitialized {
            admin: dex.admin,
            fee_numerator,
            fee_denominator,
        });
        
        Ok(())
    }
    
    pub fn create_pool(
        ctx: Context<CreatePool>,
        initial_amount_a: u64,
        initial_amount_b: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.dex_state.paused, ErrorCode::DexPaused);
        require!(initial_amount_a > 0 && initial_amount_b > 0, ErrorCode::InvalidAmounts);
        
        let pool = &mut ctx.accounts.pool;
        pool.token_a = ctx.accounts.token_a_mint.key();
        pool.token_b = ctx.accounts.token_b_mint.key();
        pool.reserve_a = initial_amount_a;
        pool.reserve_b = initial_amount_b;
        pool.k_last = (initial_amount_a as u128) * (initial_amount_b as u128);
        pool.block_timestamp_last = Clock::get()?.unix_timestamp as u64;
        
        // Transfer tokens to pool
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.pool_token_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            initial_amount_a,
        )?;
        
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.pool_token_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            initial_amount_b,
        )?;
        
        // Mint LP tokens
        let initial_liquidity = sqrt(initial_amount_a * initial_amount_b);
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.user_lp_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
                &[&[b"pool", &[ctx.bumps.pool]]],
            ),
            initial_liquidity,
        )?;
        
        emit!(PoolCreated {
            pool: pool.key(),
            token_a: pool.token_a,
            token_b: pool.token_b,
            initial_amount_a,
            initial_amount_b,
        });
        
        Ok(())
    }
    
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.dex_state.paused, ErrorCode::DexPaused);
        
        let pool = &mut ctx.accounts.pool;
        require!(!pool.locked, ErrorCode::PoolLocked);
        
        pool.locked = true;
        
        // Determine swap direction
        let (reserve_in, reserve_out) = if ctx.accounts.token_in.mint == pool.token_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };
        
        // Calculate output amount
        let fee_numerator = ctx.accounts.dex_state.fee_numerator;
        let fee_denominator = ctx.accounts.dex_state.fee_denominator;
        
        let amount_in_with_fee = amount_in * (fee_denominator - fee_numerator);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = (reserve_in * fee_denominator) + amount_in_with_fee;
        let amount_out = numerator / denominator;
        
        require!(amount_out >= min_amount_out, ErrorCode::SlippageExceeded);
        require!(amount_out < reserve_out, ErrorCode::InsufficientLiquidity);
        
        // Execute swap
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_in.to_account_info(),
                    to: ctx.accounts.pool_token_in.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.pool_token_out.to_account_info(),
                    to: ctx.accounts.user_token_out.to_account_info(),
                    authority: pool.to_account_info(),
                },
                &[&[b"pool", &[ctx.bumps.pool]]],
            ),
            amount_out,
        )?;
        
        // Update reserves
        if ctx.accounts.token_in.mint == pool.token_a {
            pool.reserve_a += amount_in;
            pool.reserve_b -= amount_out;
        } else {
            pool.reserve_b += amount_in;
            pool.reserve_a -= amount_out;
        }
        
        pool.k_last = (pool.reserve_a as u128) * (pool.reserve_b as u128);
        pool.locked = false;
        
        emit!(SwapExecuted {
            user: ctx.accounts.user.key(),
            amount_in,
            amount_out,
        });
        
        Ok(())
    }
}

// Helper functions
fn sqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

// Account structures
#[account]
pub struct DexState {
    pub admin: Pubkey,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
    pub protocol_fee_share: u64,
    pub pool_count: u64,
    pub paused: bool,
}

#[account]
pub struct Pool {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub k_last: u128,
    pub block_timestamp_last: u64,
    pub price_cumulative_a: u128,
    pub price_cumulative_b: u128,
    pub locked: bool,
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("DEX is paused")]
    DexPaused,
    #[msg("Invalid amounts")]
    InvalidAmounts,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
}
