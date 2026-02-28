use anchor_lang::prelude::*;
use switchboard_v2::VrfAccountData;

use crate::state::{Config, GameState, game_status};
use crate::error::MinesError;

/// Fulfill VRF randomness and place mines deterministically
/// This instruction consumes the Switchboard VRF result and generates mine positions
pub fn fulfill(ctx: Context<Fulfill>, vrf_randomness: [u8; 32]) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    // Verify game is still active
    require!(
        game_state.status == game_status::ACTIVE,
        MinesError::GameEnded
    );

    // Verify VRF not already fulfilled
    require!(
        game_state.vrf_fulfilled_at == 0,
        MinesError::VrfAlreadyFulfilled
    );

    // In production, verify the VRF account data here
    // For now, we accept the randomness parameter
    // In real implementation, you would:
    // 1. Verify the VRF account is valid
    // 2. Extract randomness from VrfAccountData
    // 3. Verify the callback was authorized

    // Store randomness
    game_state.vrf_randomness = Some(vrf_randomness);
    game_state.vrf_fulfilled_at = clock.unix_timestamp;

    // Generate mine positions deterministically from randomness
    let mines = generate_mine_positions(vrf_randomness, game_state.mines_count);
    
    // Store mine positions
    for (i, &mine_pos) in mines.iter().enumerate() {
        if i < 24 {
            game_state.mine_positions[i] = mine_pos;
        }
    }
    game_state.mines_placed = game_state.mines_count;

    msg!(
        "VRF fulfilled: game={}, mines_count={}, randomness={:?}",
        ctx.accounts.game_state.key(),
        game_state.mines_count,
        vrf_randomness
    );

    emit!(VrfFulfilled {
        game_state: ctx.accounts.game_state.key(),
        vrf_randomness,
        mines_count: game_state.mines_count,
    });

    Ok(())
}

/// Generate mine positions deterministically from VRF randomness
/// Uses Fisher-Yates shuffle algorithm with randomness as seed
fn generate_mine_positions(randomness: [u8; 32], count: u8) -> Vec<u8> {
    let mut positions: Vec<u8> = (0..25).collect();
    let mut result = Vec::new();
    
    // Use randomness bytes to shuffle
    let mut seed = u64::from_le_bytes([
        randomness[0], randomness[1], randomness[2], randomness[3],
        randomness[4], randomness[5], randomness[6], randomness[7],
    ]);
    
    // Fisher-Yates shuffle
    for i in 0..(count as usize) {
        // Simple LCG for pseudo-random number
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let j = i + (seed as usize % (25 - i));
        positions.swap(i, j);
    }
    
    // Take first 'count' positions as mines
    positions[..count as usize].to_vec()
}

#[derive(Accounts)]
pub struct Fulfill<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub game_state: Account<'info, GameState>,

    /// CHECK: Switchboard VRF account (verified in production)
    pub vrf_account: UncheckedAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
}

#[event]
pub struct VrfFulfilled {
    pub game_state: Pubkey,
    pub vrf_randomness: [u8; 32],
    pub mines_count: u8,
}
