# Solana Mines Game – Provably Fair Mines Betting with VRF

A production-ready, on-chain Mines casino game built on Solana using Anchor framework and Switchboard VRF (Verifiable Random Function) for provably fair mine placement. Players bet SOL or SPL tokens, choose their mine count (1-24), and reveal tiles on a 5×5 grid. Each safe reveal increases the multiplier, and players can cash out at any time to receive their payout minus the house edge.

## 🎮 Features

- **Provably Fair Gameplay**: Mine positions are determined by Switchboard VRF randomness, fully verifiable on-chain
- **Flexible Betting**: Support for SOL and SPL tokens with configurable min/max bet limits
- **Dynamic Multipliers**: Risk-adjusted multiplier progression that increases with each safe tile reveal
- **Configurable House Edge**: Adjustable house edge percentage (default 1-5%) deducted from payouts
- **Real-time Game State**: On-chain game state tracking with events for all game actions
- **Security First**: Comprehensive error handling, reentrancy protection, and checked arithmetic
- **VRF Integration**: Switchboard VRF for verifiable randomness (easily swappable to ORAO VRF)

## 🛠 Tech Stack

- **Rust**: Core program logic
- **Anchor Framework**: Solana program development framework (v0.30.1)
- **Solana**: High-performance blockchain platform
- **Switchboard VRF**: Verifiable Random Function for provable fairness
- **SPL Tokens**: Token standard for SOL and custom token support
- **TypeScript**: Test suite and client integration

## 🚀 Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor Framework](https://www.anchor-lang.com/docs/installation) (v0.30.1+)
- [Node.js](https://nodejs.org/) (v18+)
- [Yarn](https://yarnpkg.com/) or npm

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd Solana-Mines-Game
```

2. Install dependencies:
```bash
yarn install
# or
npm install
```

3. Build the program:
```bash
anchor build
```

4. Run tests:
```bash
anchor test
```

5. Deploy to localnet (optional):
```bash
anchor deploy
```

## 📖 How to Play

### Game Flow

1. **Initialize Config** (one-time, authority only):
   - Set house edge, bet limits, mine limits, and VRF parameters

2. **Start Game**:
   - Player transfers bet amount to house vault
   - Player selects number of mines (1-24)
   - Program requests VRF randomness from Switchboard

3. **Fulfill VRF**:
   - Switchboard oracle fulfills randomness request
   - Mine positions are deterministically generated from VRF seed
   - Game becomes ready for tile reveals

4. **Reveal Tiles**:
   - Player reveals tiles one by one
   - Safe tiles increase the multiplier
   - Hitting a mine ends the game (player loses bet)

5. **Cash Out**:
   - Player can cash out at any time
   - Payout = `bet_amount × multiplier × (1 - house_edge)`
   - Funds transferred from house vault to player

### Example Client Code

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MinesGame } from "../target/types/mines_game";
import { PublicKey, Keypair } from "@solana/web3.js";

// Initialize program
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.MinesGame as Program<MinesGame>;

// Start a game
const betAmount = new anchor.BN(10000000); // 0.01 SOL
const minesCount = 3;

const [gameStatePda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("game"),
    player.publicKey.toBuffer(),
    Buffer.from(Date.now().toString()),
  ],
  program.programId
);

await program.methods
  .startGame(betAmount, minesCount)
  .accounts({
    config: configPda,
    gameState: gameStatePda,
    player: player.publicKey,
    playerTokenAccount: playerTokenAccount,
    houseVault: houseVault,
    // ... other accounts
  })
  .signers([player])
  .rpc();

// After VRF fulfillment, reveal a tile
await program.methods
  .revealTile(0) // Tile index 0-24
  .accounts({
    config: configPda,
    gameState: gameStatePda,
    player: player.publicKey,
  })
  .signers([player])
  .rpc();

// Cash out
await program.methods
  .cashOut()
  .accounts({
    config: configPda,
    gameState: gameStatePda,
    player: player.publicKey,
    playerTokenAccount: playerTokenAccount,
    houseVault: houseVault,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([player])
  .rpc();
```

## 🔐 Provable Fairness

### How It Works

1. **VRF Request**: When a game starts, the program requests randomness from Switchboard VRF
2. **Randomness Generation**: Switchboard oracle generates cryptographically verifiable randomness
3. **Deterministic Mine Placement**: Mine positions are generated using a Fisher-Yates shuffle algorithm seeded with the VRF randomness
4. **Verification**: Anyone can verify mine positions by:
   - Checking the `vrf_randomness` field in the game state account
   - Running the same deterministic algorithm with the VRF seed
   - Comparing results with stored `mine_positions`

### Verifying Mine Positions

```rust
// The mine generation algorithm (from fulfill.rs):
fn generate_mine_positions(randomness: [u8; 32], count: u8) -> Vec<u8> {
    let mut positions: Vec<u8> = (0..25).collect();
    let mut seed = u64::from_le_bytes([
        randomness[0], randomness[1], randomness[2], randomness[3],
        randomness[4], randomness[5], randomness[6], randomness[7],
    ]);
    
    // Fisher-Yates shuffle
    for i in 0..(count as usize) {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let j = i + (seed as usize % (25 - i));
        positions.swap(i, j);
    }
    
    positions[..count as usize].to_vec()
}
```

### Switching to ORAO VRF

To use ORAO VRF instead of Switchboard:

1. Replace `switchboard-v2` dependency in `Cargo.toml` with `oracle-vrf`
2. Update the `fulfill` instruction to consume ORAO VRF account data
3. Modify VRF account verification logic in `fulfill.rs`
4. Update client code to use ORAO VRF queue and oracle addresses

The core mine generation algorithm remains the same - only the VRF source changes.

## 📁 Project Structure

```
Solana-Mines-Game-1/
├── programs/
│   └── mines-game/
│       └── src/
│           ├── lib.rs              # Program entry point
│           ├── state.rs             # Account structs (Config, GameState)
│           ├── error.rs             # Custom error types
│           └── instructions/
│               ├── mod.rs
│               ├── initialize.rs    # Initialize config
│               ├── start_game.rs    # Start new game
│               ├── fulfill.rs       # Fulfill VRF randomness
│               ├── reveal_tile.rs   # Reveal a tile
│               ├── cash_out.rs      # Cash out with payout
│               ├── withdraw_house.rs # Withdraw house profits
│               └── update_config.rs # Update config params
├── tests/
│   └── mines-game.ts                # Comprehensive test suite
├── Anchor.toml                      # Anchor configuration
├── Cargo.toml                       # Rust dependencies
├── package.json                     # Node.js dependencies
├── tsconfig.json                    # TypeScript configuration
└── README.md                        # This file
```

## 🧪 Testing

The test suite includes 15+ comprehensive tests covering:

- ✅ Config initialization and validation
- ✅ Game creation with bet and mine selection
- ✅ VRF fulfillment and mine placement
- ✅ Safe tile reveals and multiplier progression
- ✅ Mine hits and game loss
- ✅ Cash out at various stages
- ✅ Invalid action handling (duplicate reveals, wrong tiles, etc.)
- ✅ House edge deduction verification
- ✅ Authority-only operations
- ✅ Edge cases and error conditions

Run tests:
```bash
anchor test
```

## 🔒 Security Considerations

- **Reentrancy Protection**: All state changes happen before external calls
- **Checked Arithmetic**: All calculations use `checked_mul`, `checked_add`, etc.
- **Signer Verification**: All privileged operations verify signers
- **PDA Validation**: All PDAs are verified with proper seeds and bumps
- **Input Validation**: All user inputs are validated (bet amounts, mine counts, tile indices)
- **State Machine**: Game state transitions are strictly enforced
- **VRF Verification**: VRF randomness is verified before use (in production)

### Audit Recommendation

⚠️ **This code has not been audited.** Before deploying to mainnet, we strongly recommend:
- Professional security audit by a reputable Solana auditing firm
- Comprehensive penetration testing
- Economic model review
- VRF integration verification
- Load testing under high transaction volume

## 📊 Multiplier Formula

The multiplier increases with each safe tile reveal using a risk-adjusted geometric progression:

```
multiplier = base_multiplier ^ revealed_count

where:
  base_multiplier = 1.0 + (risk_factor × 0.15)
  risk_factor = (mines_count / total_tiles) × 2.0 + 1.0
```

This ensures:
- Higher mine count = higher risk = higher potential multiplier per reveal
- Multiplier caps at 100x (1,000,000 basis points) to prevent overflow
- Fair risk/reward balance aligned with real Mines casino games

## 🎯 Instructions Reference

### `initialize`
Initialize global configuration (authority only)
- Parameters: house edge, bet limits, mine limits, VRF keys

### `start_game`
Start a new Mines game
- Parameters: bet amount, mines count
- Transfers bet to house vault
- Requests VRF randomness

### `fulfill`
Fulfill VRF randomness and place mines
- Parameters: VRF randomness (32 bytes)
- Generates mine positions deterministically

### `reveal_tile`
Reveal a tile on the grid
- Parameters: tile index (0-24)
- Updates multiplier if safe, ends game if mine

### `cash_out`
Cash out and receive payout
- Calculates: `bet × multiplier × (1 - house_edge)`
- Transfers funds to player

### `withdraw_house`
Withdraw house profits (authority only)
- Parameters: withdrawal amount

### `update_config`
Update configuration (authority only)
- Parameters: optional updates to any config field

## 📝 Events

The program emits Anchor events for all major actions:

- `GameStarted`: When a new game begins
- `VrfFulfilled`: When VRF randomness is received
- `TileRevealed`: When a safe tile is revealed
- `MineHit`: When a mine is hit (game loss)
- `CashOutEvent`: When a player cashes out

## 🤝 Contact

- telegram: https://t.me/CasinoCutup
- twitter:  https://x.com/CasinoCutup
