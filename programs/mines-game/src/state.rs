use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

/// Global configuration account for the Mines game
#[account]
pub struct Config {
    /// Authority that can update config and withdraw house funds
    pub authority: Pubkey,
    /// House edge percentage (basis points, e.g., 500 = 5%)
    pub house_edge_bps: u16,
    /// Minimum bet amount (in lamports or token smallest unit)
    pub min_bet: u64,
    /// Maximum bet amount (in lamports or token smallest unit)
    pub max_bet: u64,
    /// Minimum number of mines (typically 1)
    pub min_mines: u8,
    /// Maximum number of mines (typically 24)
    pub max_mines: u8,
    /// Switchboard VRF queue public key
    pub vrf_queue: Pubkey,
    /// Switchboard VRF oracle key
    pub vrf_oracle: Pubkey,
    /// Fee wallet for VRF requests
    pub fee_wallet: Pubkey,
    /// House vault for SOL bets
    pub house_vault: Pubkey,
    /// Bump seed for config PDA
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        2 + // house_edge_bps
        8 + // min_bet
        8 + // max_bet
        1 + // min_mines
        1 + // max_mines
        32 + // vrf_queue
        32 + // vrf_oracle
        32 + // fee_wallet
        32 + // house_vault
        1; // bump
}

/// Game state account for a single Mines game instance
#[account]
pub struct GameState {
    /// Player who started the game
    pub player: Pubkey,
    /// Bet amount (in lamports or token smallest unit)
    pub bet_amount: u64,
    /// Number of mines selected by player
    pub mines_count: u8,
    /// Switchboard VRF request ID
    pub vrf_request_id: Option<[u8; 32]>,
    /// Bitmap of revealed tiles (25 bits, one per tile)
    pub revealed_tiles: u32,
    /// Current multiplier (in basis points, e.g., 10000 = 1.0x)
    pub current_multiplier: u64,
    /// Game status: 0 = Active, 1 = CashedOut, 2 = Lost, 3 = Expired
    pub status: u8,
    /// Mine positions (indices 0-24) - set after VRF fulfillment
    pub mine_positions: [u8; 24],
    /// Number of mines actually placed (should equal mines_count)
    pub mines_placed: u8,
    /// Timestamp when game was created
    pub created_at: i64,
    /// Timestamp when VRF was fulfilled (0 if not fulfilled)
    pub vrf_fulfilled_at: i64,
    /// VRF randomness seed (from Switchboard)
    pub vrf_randomness: Option<[u8; 32]>,
    /// Bump seed for game state PDA
    pub bump: u8,
}

impl GameState {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // bet_amount
        1 + // mines_count
        1 + 32 + // vrf_request_id (Option)
        4 + // revealed_tiles (u32 bitmap)
        8 + // current_multiplier
        1 + // status
        24 + // mine_positions array
        1 + // mines_placed
        8 + // created_at
        8 + // vrf_fulfilled_at
        1 + 32 + // vrf_randomness (Option)
        1; // bump

    /// Check if a tile is revealed
    pub fn is_tile_revealed(&self, tile_index: u8) -> bool {
        if tile_index >= 25 {
            return false;
        }
        (self.revealed_tiles >> tile_index) & 1 == 1
    }

    /// Mark a tile as revealed
    pub fn reveal_tile(&mut self, tile_index: u8) {
        if tile_index < 25 {
            self.revealed_tiles |= 1u32 << tile_index;
        }
    }

    /// Check if a tile is a mine
    pub fn is_mine(&self, tile_index: u8) -> bool {
        self.mine_positions[..self.mines_placed as usize]
            .contains(&tile_index)
    }

    /// Count number of revealed tiles
    pub fn revealed_count(&self) -> u8 {
        self.revealed_tiles.count_ones() as u8
    }
}

/// Game status constants
pub mod game_status {
    pub const ACTIVE: u8 = 0;
    pub const CASHED_OUT: u8 = 1;
    pub const LOST: u8 = 2;
    pub const EXPIRED: u8 = 3;
}
