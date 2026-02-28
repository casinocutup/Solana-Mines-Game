use anchor_lang::prelude::*;

#[error_code]
pub enum MinesError {
    #[msg("Invalid number of mines. Must be between 1 and 24.")]
    InvalidMinesCount,

    #[msg("Invalid tile index. Must be between 0 and 24.")]
    InvalidTileIndex,

    #[msg("Tile already revealed.")]
    TileAlreadyRevealed,

    #[msg("Game not ready. VRF not fulfilled yet.")]
    GameNotReady,

    #[msg("Game already ended.")]
    GameEnded,

    #[msg("Game still active. Cannot perform this action.")]
    GameStillActive,

    #[msg("Bet amount below minimum.")]
    BetTooLow,

    #[msg("Bet amount above maximum.")]
    BetTooHigh,

    #[msg("Insufficient funds in house vault.")]
    InsufficientVaultFunds,

    #[msg("VRF request expired. Game timed out.")]
    VrfRequestExpired,

    #[msg("Invalid VRF fulfillment. Proof verification failed.")]
    InvalidVrfFulfillment,

    #[msg("Unauthorized. Only house authority can perform this action.")]
    Unauthorized,

    #[msg("Invalid house edge. Must be between 0 and 100.")]
    InvalidHouseEdge,

    #[msg("Cannot cash out with zero multiplier.")]
    ZeroMultiplier,

    #[msg("Invalid game state transition.")]
    InvalidGameState,

    #[msg("VRF already fulfilled for this game.")]
    VrfAlreadyFulfilled,
}
