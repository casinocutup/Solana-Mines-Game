use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{Config, GameState, game_status};
use crate::error::MinesError;

/// Cash out and receive payout based on current multiplier
pub fn cash_out(ctx: Context<CashOut>) -> Result<()> {
    let config = &ctx.accounts.config;
    let game_state = &mut ctx.accounts.game_state;

    // Verify game is active
    require!(
        game_state.status == game_status::ACTIVE,
        MinesError::GameEnded
    );

    // Verify VRF has been fulfilled
    require!(
        game_state.vrf_fulfilled_at > 0,
        MinesError::GameNotReady
    );

    // Verify multiplier is valid
    require!(
        game_state.current_multiplier > 0,
        MinesError::ZeroMultiplier
    );

    // Calculate payout: multiplier * bet_amount * (1 - house_edge)
    let gross_payout = game_state.bet_amount
        .checked_mul(game_state.current_multiplier)
        .ok_or(MinesError::InvalidGameState)?
        .checked_div(10000) // Divide by 10000 to convert from basis points
        .ok_or(MinesError::InvalidGameState)?;

    let house_edge_amount = gross_payout
        .checked_mul(config.house_edge_bps as u64)
        .ok_or(MinesError::InvalidGameState)?
        .checked_div(10000)
        .ok_or(MinesError::InvalidGameState)?;

    let net_payout = gross_payout
        .checked_sub(house_edge_amount)
        .ok_or(MinesError::InvalidGameState)?;

    // Verify vault has sufficient funds
    require!(
        ctx.accounts.house_vault.amount >= net_payout,
        MinesError::InsufficientVaultFunds
    );

    // Transfer payout to player
    // Note: In production, house_vault should be a PDA owned by the config
    // For now, we assume the house_vault is a token account owned by the config PDA
    let seeds = &[
        b"config",
        &[config.bump],
    ];
    let signer = &[&seeds[..]];

    // Get the authority PDA for the house vault
    // In a real implementation, you'd derive this from config
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.house_vault.to_account_info(),
            to: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, net_payout)?;

    // Update game state
    game_state.status = game_status::CASHED_OUT;

    msg!(
        "Cash out successful: player={}, payout={}, multiplier={}",
        game_state.player,
        net_payout,
        game_state.current_multiplier
    );

    emit!(CashOutEvent {
        game_state: ctx.accounts.game_state.key(),
        player: game_state.player,
        payout: net_payout,
        multiplier: game_state.current_multiplier,
        revealed_count: game_state.revealed_count(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CashOut<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        has_one = player @ MinesError::Unauthorized
    )]
    pub game_state: Account<'info, GameState>,

    pub player: Signer<'info>,

    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: House vault token account (authority is config PDA)
    pub house_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[event]
pub struct CashOutEvent {
    pub game_state: Pubkey,
    pub player: Pubkey,
    pub payout: u64,
    pub multiplier: u64,
    pub revealed_count: u8,
}
