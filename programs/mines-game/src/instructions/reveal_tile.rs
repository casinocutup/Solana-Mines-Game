use anchor_lang::prelude::*;

use crate::state::{Config, GameState, game_status};
use crate::error::MinesError;

/// Reveal a tile - if safe, increase multiplier; if mine, lose bet
pub fn reveal_tile(ctx: Context<RevealTile>, tile_index: u8) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    // Validate tile index
    require!(tile_index < 25, MinesError::InvalidTileIndex);

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

    // Check if tile already revealed
    require!(
        !game_state.is_tile_revealed(tile_index),
        MinesError::TileAlreadyRevealed
    );

    // Check if it's a mine
    if game_state.is_mine(tile_index) {
        // Game over - player loses
        game_state.status = game_status::LOST;
        
        msg!(
            "Mine hit! Game lost: player={}, tile={}",
            game_state.player,
            tile_index
        );

        emit!(MineHit {
            game_state: ctx.accounts.game_state.key(),
            player: game_state.player,
            tile_index,
        });

        return Ok(());
    }

    // Safe tile - reveal it and update multiplier
    game_state.reveal_tile(tile_index);
    let revealed_count = game_state.revealed_count();
    
    // Calculate new multiplier based on Mines formula
    // Formula: multiplier = (total_tiles / (total_tiles - mines)) ^ revealed_safe_tiles
    // This is a simplified version; real casinos use more complex risk-adjusted formulas
    let total_tiles = 25u64;
    let safe_tiles = total_tiles - game_state.mines_count as u64;
    let base = (total_tiles * 10000) / safe_tiles; // Multiply by 10000 for precision
    
    // Geometric progression: each safe reveal multiplies by base
    // More accurate formula: multiplier increases exponentially with risk
    let multiplier = calculate_multiplier(
        revealed_count,
        game_state.mines_count,
        total_tiles as u8,
    );
    
    game_state.current_multiplier = multiplier;

    msg!(
        "Safe tile revealed: tile={}, revealed={}, multiplier={}",
        tile_index,
        revealed_count,
        multiplier
    );

    emit!(TileRevealed {
        game_state: ctx.accounts.game_state.key(),
        player: game_state.player,
        tile_index,
        revealed_count,
        multiplier,
    });

    Ok(())
}

/// Calculate multiplier based on revealed safe tiles
/// Uses risk-adjusted geometric progression formula
/// Formula inspired by real Mines games: higher risk (more mines) = higher potential multiplier
fn calculate_multiplier(revealed_count: u8, mines_count: u8, total_tiles: u8) -> u64 {
    if revealed_count == 0 {
        return 10000; // 1.0x base
    }

    let safe_tiles = total_tiles - mines_count;
    let remaining_safe = safe_tiles - revealed_count;
    
    if remaining_safe == 0 {
        return 10000; // All safe tiles revealed, but can't go higher
    }

    // Risk factor: more mines = higher risk = higher multiplier per reveal
    let risk_factor = (mines_count as f64 / total_tiles as f64) * 2.0 + 1.0;
    
    // Base multiplier per reveal (increases with risk)
    let base_multiplier = 1.0 + (risk_factor * 0.15); // 15% base increase, scaled by risk
    
    // Calculate: (base_multiplier ^ revealed_count) * 10000 for basis points
    let multiplier = base_multiplier.powi(revealed_count as i32);
    
    // Cap at reasonable maximum (100x = 1,000,000 basis points)
    let result = (multiplier * 10000.0) as u64;
    result.min(1_000_000)
}

#[derive(Accounts)]
pub struct RevealTile<'info> {
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
}

#[event]
pub struct TileRevealed {
    pub game_state: Pubkey,
    pub player: Pubkey,
    pub tile_index: u8,
    pub revealed_count: u8,
    pub multiplier: u64,
}

#[event]
pub struct MineHit {
    pub game_state: Pubkey,
    pub player: Pubkey,
    pub tile_index: u8,
}
