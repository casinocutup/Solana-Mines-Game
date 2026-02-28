use anchor_lang::prelude::*;

pub mod error;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod mines_game {
    use super::*;

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
        instructions::initialize::initialize(
            ctx,
            house_edge_bps,
            min_bet,
            max_bet,
            min_mines,
            max_mines,
            vrf_queue,
            vrf_oracle,
        )
    }

    /// Start a new Mines game by placing a bet and requesting VRF randomness
    pub fn start_game(
        ctx: Context<StartGame>,
        bet_amount: u64,
        mines_count: u8,
    ) -> Result<()> {
        instructions::start_game::start_game(ctx, bet_amount, mines_count)
    }

    /// Fulfill VRF randomness and place mines deterministically
    /// Note: In production, this would be called by Switchboard's callback
    /// For testing, we allow manual fulfillment with verified randomness
    pub fn fulfill(ctx: Context<Fulfill>, vrf_randomness: [u8; 32]) -> Result<()> {
        instructions::fulfill::fulfill(ctx, vrf_randomness)
    }

    /// Reveal a tile - if safe, increase multiplier; if mine, lose bet
    pub fn reveal_tile(ctx: Context<RevealTile>, tile_index: u8) -> Result<()> {
        instructions::reveal_tile::reveal_tile(ctx, tile_index)
    }

    /// Cash out and receive payout based on current multiplier
    pub fn cash_out(ctx: Context<CashOut>) -> Result<()> {
        instructions::cash_out::cash_out(ctx)
    }

    /// Withdraw accumulated house profits (authority only)
    pub fn withdraw_house(ctx: Context<WithdrawHouse>, amount: u64) -> Result<()> {
        instructions::withdraw_house::withdraw_house(ctx, amount)
    }

    /// Update configuration parameters (authority only)
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        house_edge_bps: Option<u16>,
        min_bet: Option<u64>,
        max_bet: Option<u64>,
        min_mines: Option<u8>,
        max_mines: Option<u8>,
        vrf_queue: Option<Pubkey>,
        vrf_oracle: Option<Pubkey>,
    ) -> Result<()> {
        instructions::update_config::update_config(
            ctx,
            house_edge_bps,
            min_bet,
            max_bet,
            min_mines,
            max_mines,
            vrf_queue,
            vrf_oracle,
        )
    }
}
