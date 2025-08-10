use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod cross_chain_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, decimals: u8) -> Result<()> {
        let token_state = &mut ctx.accounts.token_state;
        token_state.authority = ctx.accounts.authority.key();
        token_state.total_supply = 0;
        token_state.decimals = decimals;
        token_state.is_initialized = true;
        Ok(())
    }

    pub fn mint(ctx: Context<Mint>, amount: u64) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.token_state.authority,
            ErrorCode::Unauthorized
        );

        let token_state = &mut ctx.accounts.token_state;
        let user_balance = &mut ctx.accounts.user_balance;

        token_state.total_supply = token_state
            .total_supply
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        user_balance.amount = user_balance
            .amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        emit!(MintEvent {
            to: ctx.accounts.to.key(),
            amount,
            total_supply: token_state.total_supply,
        });

        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let from_balance = &mut ctx.accounts.from_balance;
        let to_balance = &mut ctx.accounts.to_balance;

        require!(
            from_balance.amount >= amount,
            ErrorCode::InsufficientBalance
        );

        from_balance.amount = from_balance
            .amount
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        to_balance.amount = to_balance
            .amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        emit!(TransferEvent {
            from: ctx.accounts.from.key(),
            to: ctx.accounts.to.key(),
            amount,
        });

        Ok(())
    }

    pub fn lock_for_bridge(
        ctx: Context<LockForBridge>,
        amount: u64,
        target_chain: u32,
    ) -> Result<()> {
        let user_balance = &mut ctx.accounts.user_balance;
        let bridge_vault = &mut ctx.accounts.bridge_vault;

        require!(
            user_balance.amount >= amount,
            ErrorCode::InsufficientBalance
        );

        user_balance.amount = user_balance
            .amount
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        bridge_vault.locked_amount = bridge_vault
            .locked_amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        emit!(CrossChainLockEvent {
            from: ctx.accounts.user.key(),
            amount,
            target_chain,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn receive_from_bridge(
        ctx: Context<ReceiveFromBridge>,
        amount: u64,
        source_chain: u32,
    ) -> Result<()> {
        require!(
            ctx.accounts.bridge_authority.key() == ctx.accounts.bridge_vault.authority,
            ErrorCode::Unauthorized
        );

        let bridge_vault = &mut ctx.accounts.bridge_vault;
        let user_balance = &mut ctx.accounts.user_balance;

        require!(
            bridge_vault.locked_amount >= amount,
            ErrorCode::InsufficientVaultBalance
        );

        bridge_vault.locked_amount = bridge_vault
            .locked_amount
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        user_balance.amount = user_balance
            .amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        emit!(CrossChainReceiveEvent {
            to: ctx.accounts.to.key(),
            amount,
            source_chain,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TokenState::LEN
    )]
    pub token_state: Account<'info, TokenState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(mut)]
    pub token_state: Account<'info, TokenState>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + UserBalance::LEN,
        seeds = [b"balance", to.key().as_ref()],
        bump
    )]
    pub user_balance: Account<'info, UserBalance>,
    pub to: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(
        mut,
        seeds = [b"balance", from.key().as_ref()],
        bump
    )]
    pub from_balance: Account<'info, UserBalance>,
    #[account(
        init_if_needed,
        payer = from,
        space = 8 + UserBalance::LEN,
        seeds = [b"balance", to.key().as_ref()],
        bump
    )]
    pub to_balance: Account<'info, UserBalance>,
    pub from: Signer<'info>,
    pub to: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockForBridge<'info> {
    #[account(
        mut,
        seeds = [b"balance", user.key().as_ref()],
        bump
    )]
    pub user_balance: Account<'info, UserBalance>,
    #[account(
        mut,
        seeds = [b"bridge_vault"],
        bump
    )]
    pub bridge_vault: Account<'info, BridgeVault>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReceiveFromBridge<'info> {
    #[account(
        mut,
        seeds = [b"bridge_vault"],
        bump
    )]
    pub bridge_vault: Account<'info, BridgeVault>,
    #[account(
        init_if_needed,
        payer = bridge_authority,
        space = 8 + UserBalance::LEN,
        seeds = [b"balance", to.key().as_ref()],
        bump
    )]
    pub user_balance: Account<'info, UserBalance>,
    pub to: AccountInfo<'info>,
    #[account(mut)]
    pub bridge_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct TokenState {
    pub authority: Pubkey,
    pub total_supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
}

impl TokenState {
    pub const LEN: usize = 32 + 8 + 1 + 1;
}

#[account]
pub struct UserBalance {
    pub owner: Pubkey,
    pub amount: u64,
}

impl UserBalance {
    pub const LEN: usize = 32 + 8;
}

#[account]
pub struct BridgeVault {
    pub authority: Pubkey,
    pub locked_amount: u64,
}

impl BridgeVault {
    pub const LEN: usize = 32 + 8;
}

#[event]
pub struct MintEvent {
    pub to: Pubkey,
    pub amount: u64,
    pub total_supply: u64,
}

#[event]
pub struct TransferEvent {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CrossChainLockEvent {
    pub from: Pubkey,
    pub amount: u64,
    pub target_chain: u32,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainReceiveEvent {
    pub to: Pubkey,
    pub amount: u64,
    pub source_chain: u32,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
}