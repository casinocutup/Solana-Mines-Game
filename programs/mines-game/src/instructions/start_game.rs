use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{Config, GameState, game_status};
use crate::error::MinesError;

/// Start a new Mines game by placing a bet and requesting VRF randomness
pub fn start_game(
    ctx: Context<StartGame>,
    bet_amount: u64,
    mines_count: u8,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    // Validate bet amount
    require!(
        bet_amount >= config.min_bet && bet_amount <= config.max_bet,
        MinesError::BetTooLow
    );

    // Validate mines count
    require!(
        mines_count >= config.min_mines && mines_count <= config.max_mines,
        MinesError::InvalidMinesCount
    );

    // Transfer bet to house vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.house_vault.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, bet_amount)?;

    // Initialize game state
    game_state.player = ctx.accounts.player.key();
    game_state.bet_amount = bet_amount;
    game_state.mines_count = mines_count;
    game_state.vrf_request_id = None;
    game_state.revealed_tiles = 0;
    game_state.current_multiplier = 10000; // Start at 1.0x (10000 basis points)
    game_state.status = game_status::ACTIVE;
    game_state.mine_positions = [255; 24]; // Initialize with invalid values
    game_state.mines_placed = 0;
    game_state.created_at = clock.unix_timestamp;
    game_state.vrf_fulfilled_at = 0;
    game_state.vrf_randomness = None;
    game_state.bump = ctx.bumps.game_state;

    // Request VRF randomness from Switchboard
    // Note: In production, you would use Switchboard's VRF CPI here
    // For now, we'll store a placeholder and handle fulfillment separately
    // The actual VRF request would be done via Switchboard's program

    msg!(
        "Game started: player={}, bet={}, mines={}",
        ctx.accounts.player.key(),
        bet_amount,
        mines_count
    );

    emit!(GameStarted {
        player: ctx.accounts.player.key(),
        bet_amount,
        mines_count,
        game_state: ctx.accounts.game_state.key(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct StartGame<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = player,
        space = GameState::LEN,
        seeds = [
            b"game",
            player.key().as_ref(),
            nonce_seed.key().as_ref(),
        ],
        bump
    )]
    pub game_state: Account<'info, GameState>,
    
    /// CHECK: Nonce seed account for uniqueness (can be any account, typically a new keypair)
    pub nonce_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub player: Signer<'info>,

    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: House vault token account
    pub house_vault: UncheckedAccount<'info>,

    /// CHECK: Switchboard VRF queue
    pub vrf_queue: UncheckedAccount<'info>,

    /// CHECK: Switchboard VRF oracle
    pub vrf_oracle: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[event]
pub struct GameStarted {
    pub player: Pubkey,
    pub bet_amount: u64,
    pub mines_count: u8,
    pub game_state: Pubkey,
}
