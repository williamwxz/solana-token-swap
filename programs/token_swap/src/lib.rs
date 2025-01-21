use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Token, Transfer},
};

declare_id!("F7T2naGX3R3izoV96t84788Wru2foc8csf7AvcTwuCH4");

#[program]
pub mod token_swap {
    use super::*;

    /// Initialize the token swap pool
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee: u64, // Fee (e.g., 0.3%)
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        pool.token_a_vault = ctx.accounts.token_a_vault.key();
        pool.token_b_vault = ctx.accounts.token_b_vault.key();
        pool.authority = ctx.accounts.authority.key();
        pool.fee = fee;

        Ok(())
    }

    /// Deposit tokens into the pool
    pub fn deposit(
        ctx: Context<Deposit>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        // Transfer Token A from user to pool
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.token_a_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_a)?;

        // Transfer Token B from user to pool
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_b.to_account_info(),
                to: ctx.accounts.token_b_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_b)?;

        Ok(())
    }

    /// Perform a token swap
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;

        // Determine the vaults for Token A and Token B
        let (vault_in, vault_out) = if ctx.accounts.token_a_vault.key() == ctx.accounts.source_vault.key() {
            (&ctx.accounts.token_a_vault, &ctx.accounts.token_b_vault)
        } else {
            (&ctx.accounts.token_b_vault, &ctx.accounts.token_a_vault)
        };

        // Calculate the output amount using a simple constant-product AMM formula
        let reserve_in = vault_in.amount;
        let reserve_out = vault_out.amount;

        // Include a fee (e.g., 0.3%)
        let amount_in_with_fee = amount_in * (1000 - pool.fee) / 1000;
        let amount_out = (amount_in_with_fee * reserve_out) / (reserve_in + amount_in_with_fee);

        // Ensure the amount_out is greater than or equal to the minimum amount expected
        require!(amount_out >= min_amount_out, ErrorCode::SlippageExceeded);

        // Transfer tokens
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.source_vault.to_account_info(),
                to: ctx.accounts.destination_vault.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_out)?;

        Ok(())
    }

    /// Withdraw tokens from the pool
    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        // Transfer Token A from pool to user
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_a_vault.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_a)?;

        // Transfer Token B from pool to user
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_b_vault.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_b)?;

        Ok(())
    }
}

#[account]
pub struct Pool {
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub authority: Pubkey,
    pub fee: u64, // Fee in basis points (e.g., 30 = 0.3%)
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = user, space = 8 + 32 * 3 + 8)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_a_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, token::TokenAccount>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_a_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub user_token_a: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_a_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub source_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub destination_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub pool_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_a_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub user_token_a: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub pool_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The output amount is less than the minimum required due to slippage.")]
    SlippageExceeded,
}