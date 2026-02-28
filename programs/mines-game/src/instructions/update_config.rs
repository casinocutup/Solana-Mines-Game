use anchor_lang::prelude::*;

use crate::state::Config;
use crate::error::MinesError;

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
    let config = &mut ctx.accounts.config;

    // Verify authority
    require!(
        ctx.accounts.authority.key() == config.authority,
        MinesError::Unauthorized
    );

    // Update fields if provided
    if let Some(edge) = house_edge_bps {
        require!(edge <= 10000, MinesError::InvalidHouseEdge);
        config.house_edge_bps = edge;
    }

    if let Some(min) = min_bet {
        require!(min > 0, MinesError::BetTooLow);
        config.min_bet = min;
    }

    if let Some(max) = max_bet {
        require!(max >= config.min_bet, MinesError::BetTooHigh);
        config.max_bet = max;
    }

    if let Some(min) = min_mines {
        require!(min >= 1 && min <= 24, MinesError::InvalidMinesCount);
        config.min_mines = min;
    }

    if let Some(max) = max_mines {
        require!(max >= config.min_mines && max <= 24, MinesError::InvalidMinesCount);
        config.max_mines = max;
    }

    if let Some(queue) = vrf_queue {
        config.vrf_queue = queue;
    }

    if let Some(oracle) = vrf_oracle {
        config.vrf_oracle = oracle;
    }

    msg!("Config updated by authority: {}", ctx.accounts.authority.key());

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    pub authority: Signer<'info>,
}
