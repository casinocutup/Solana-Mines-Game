use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::state::Config;
use crate::error::MinesError;

/// Initialize the global configuration for the Mines game
pub fn initialize(
    ctx: Context<Initialize>,
    house_edge_bps: u16,
    min_bet: u64,
    max_bet: u64,
    min_mines: u8,
    max_mines: u8,
    vrf_queue: Pubkey,
    vrf_oracle: Pubkey,
) -> Result<()> {
    // Validate house edge (0-10000 basis points = 0-100%)
    require!(
        house_edge_bps <= 10000,
        MinesError::InvalidHouseEdge
    );

    // Validate bet limits
    require!(min_bet > 0, MinesError::BetTooLow);
    require!(max_bet >= min_bet, MinesError::BetTooLow);

    // Validate mines limits
    require!(min_mines >= 1, MinesError::InvalidMinesCount);
    require!(max_mines <= 24, MinesError::InvalidMinesCount);
    require!(max_mines >= min_mines, MinesError::InvalidMinesCount);

    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.house_edge_bps = house_edge_bps;
    config.min_bet = min_bet;
    config.max_bet = max_bet;
    config.min_mines = min_mines;
    config.max_mines = max_mines;
    config.vrf_queue = vrf_queue;
    config.vrf_oracle = vrf_oracle;
    config.fee_wallet = ctx.accounts.fee_wallet.key();
    config.house_vault = ctx.accounts.house_vault.key();
    config.bump = ctx.bumps.config;

    msg!("Config initialized with house edge: {} bps", house_edge_bps);

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Fee wallet for VRF requests (can be any account)
    pub fee_wallet: UncheckedAccount<'info>,

    /// CHECK: House vault for holding bets (can be any account, typically a PDA)
    pub house_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
