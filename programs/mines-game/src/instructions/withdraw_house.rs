use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::Config;
use crate::error::MinesError;

/// Withdraw accumulated house profits (authority only)
pub fn withdraw_house(ctx: Context<WithdrawHouse>, amount: u64) -> Result<()> {
    let config = &ctx.accounts.config;

    // Verify authority
    require!(
        ctx.accounts.authority.key() == config.authority,
        MinesError::Unauthorized
    );

    // Verify sufficient balance
    require!(
        ctx.accounts.house_vault.amount >= amount,
        MinesError::InsufficientVaultFunds
    );

    // Transfer to authority
    let seeds = &[
        b"config",
        &[config.bump],
    ];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.house_vault.to_account_info(),
            to: ctx.accounts.authority_token_account.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, amount)?;

    msg!(
        "House withdrawal: authority={}, amount={}",
        ctx.accounts.authority.key(),
        amount
    );

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawHouse<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    pub authority: Signer<'info>,

    #[account(mut)]
    /// CHECK: House vault token account (authority is config PDA)
    pub house_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
