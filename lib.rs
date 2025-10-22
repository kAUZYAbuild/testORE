use anchor_lang::prelude::*;
use sha3::{Digest, Keccak256};

declare_id!("TESTORE11111111111111111111111111111111111");

/// TestORE - Solana Testnet Mining Program
/// 
/// This program implements proof-of-work mining on Solana testnet,
/// tracking miner statistics for mainnet airdrop rewards.
/// 
/// Based on the ORE protocol, modified for testnet stress testing.
#[program]
pub mod testore_program {
    use super::*;

    /// Initialize a new miner account
    /// 
    /// Creates a PDA to track mining statistics for the caller.
    /// Each wallet can have one miner account.
    pub fn initialize_miner(ctx: Context<InitializeMiner>) -> Result<()> {
        let miner = &mut ctx.accounts.miner;
        miner.authority = ctx.accounts.authority.key();
        miner.total_hashes = 0;
        miner.rounds_completed = 0;
        miner.last_hash_at = Clock::get()?.unix_timestamp;
        miner.current_streak = 0;
        miner.best_difficulty = 0;
        miner.bump = ctx.bumps.miner;
        
        msg!("✅ Miner initialized: {}", miner.authority);
        Ok(())
    }

    /// Submit a proof of work
    /// 
    /// Validates that the hash meets difficulty requirements and updates
    /// miner statistics. Rate limited to 1 submission per second.
    pub fn submit_proof(
        ctx: Context<SubmitProof>,
        nonce: u64,
        difficulty: u8,
    ) -> Result<()> {
        let miner = &mut ctx.accounts.miner;
        let global_round = &mut ctx.accounts.global_round;
        let clock = Clock::get()?;

        // Verify the proof
        let hash = hash_proof(
            &miner.authority,
            &global_round.current_challenge,
            nonce,
        );

        // Check if hash meets difficulty requirement
        require!(
            check_difficulty(&hash, difficulty),
            ErrorCode::InsufficientDifficulty
        );

        // Ensure minimum time between submissions (anti-spam)
        require!(
            clock.unix_timestamp - miner.last_hash_at >= 1,
            ErrorCode::TooManySubmissions
        );

        // Require minimum difficulty
        require!(
            difficulty >= global_round.min_difficulty,
            ErrorCode::DifficultyTooLow
        );

        // Update miner stats
        miner.total_hashes = miner.total_hashes.checked_add(1).unwrap();
        miner.last_hash_at = clock.unix_timestamp;
        miner.current_streak = miner.current_streak.checked_add(1).unwrap();
        
        if difficulty > miner.best_difficulty {
            miner.best_difficulty = difficulty;
        }

        // Check if round completed (10 consecutive hashes)
        if miner.current_streak >= 10 {
            miner.rounds_completed = miner.rounds_completed.checked_add(1).unwrap();
            miner.current_streak = 0;
            
            // Update global round
            global_round.total_rounds_completed = global_round
                .total_rounds_completed
                .checked_add(1)
                .unwrap();
        }

        // Update global stats
        global_round.total_hashes_submitted = global_round
            .total_hashes_submitted
            .checked_add(1)
            .unwrap();

        msg!(
            "⛏️ Proof accepted - Hashes: {}, Rounds: {}, Difficulty: {}",
            miner.total_hashes,
            miner.rounds_completed,
            difficulty
        );

        Ok(())
    }

    /// Initialize the global round state
    /// 
    /// Sets up the initial mining challenge and parameters.
    /// Should be called once during deployment.
    pub fn initialize_global_round(
        ctx: Context<InitializeGlobalRound>,
        admin: Pubkey,
    ) -> Result<()> {
        let global_round = &mut ctx.accounts.global_round;
        let clock = Clock::get()?;

        // Generate initial challenge
        let challenge = generate_challenge(&clock);
        
        global_round.current_challenge = challenge;
        global_round.round_number = 1;
        global_round.started_at = clock.unix_timestamp;
        global_round.min_difficulty = 8; // Testnet: easier than mainnet
        global_round.total_hashes_submitted = 0;
        global_round.total_rounds_completed = 0;
        global_round.admin = admin;
        global_round.bump = ctx.bumps.global_round;

        msg!("🌍 Global round initialized - Challenge generated");
        Ok(())
    }

    /// Rotate to a new mining round
    /// 
    /// Admin-only function to update the challenge and adjust difficulty.
    pub fn rotate_round(ctx: Context<RotateRound>) -> Result<()> {
        let global_round = &mut ctx.accounts.global_round;
        let clock = Clock::get()?;

        // Generate new challenge
        global_round.current_challenge = generate_challenge(&clock);
        global_round.round_number = global_round.round_number.checked_add(1).unwrap();
        global_round.started_at = clock.unix_timestamp;
        
        // Dynamic difficulty adjustment
        // If too many submissions, increase difficulty
        if global_round.total_hashes_submitted > 1_000_000 {
            global_round.min_difficulty = global_round
                .min_difficulty
                .saturating_add(1)
                .min(16); // Cap at 16 bits
        }

        // Reset counters
        global_round.total_hashes_submitted = 0;

        msg!("🔄 Round rotated to #{}", global_round.round_number);
        Ok(())
    }
}

// ============================================================================
// Account Contexts
// ============================================================================

#[derive(Accounts)]
pub struct InitializeMiner<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Miner::INIT_SPACE,
        seeds = [b"miner", authority.key().as_ref()],
        bump
    )]
    pub miner: Account<'info, Miner>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitProof<'info> {
    #[account(
        mut,
        seeds = [b"miner", authority.key().as_ref()],
        bump = miner.bump,
        has_one = authority
    )]
    pub miner: Account<'info, Miner>,
    
    #[account(
        mut,
        seeds = [b"global_round"],
        bump = global_round.bump
    )]
    pub global_round: Account<'info, GlobalRound>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeGlobalRound<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GlobalRound::INIT_SPACE,
        seeds = [b"global_round"],
        bump
    )]
    pub global_round: Account<'info, GlobalRound>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RotateRound<'info> {
    #[account(
        mut,
        seeds = [b"global_round"],
        bump = global_round.bump,
        has_one = admin
    )]
    pub global_round: Account<'info, GlobalRound>,
    
    pub admin: Signer<'info>,
}

// ============================================================================
// Account Data Structures
// ============================================================================

/// Stores individual miner statistics
#[account]
#[derive(InitSpace)]
pub struct Miner {
    /// Wallet address of the miner
    pub authority: Pubkey,
    
    /// Total number of valid hashes submitted
    pub total_hashes: u64,
    
    /// Number of completed rounds (10 consecutive hashes)
    pub rounds_completed: u32,
    
    /// Unix timestamp of last hash submission
    pub last_hash_at: i64,
    
    /// Current consecutive hash streak
    pub current_streak: u32,
    
    /// Highest difficulty achieved
    pub best_difficulty: u8,
    
    /// PDA bump seed
    pub bump: u8,
}

/// Global mining round state
#[account]
#[derive(InitSpace)]
pub struct GlobalRound {
    /// Current mining challenge (32 bytes)
    pub current_challenge: [u8; 32],
    
    /// Round number (increments on rotation)
    pub round_number: u64,
    
    /// Unix timestamp when round started
    pub started_at: i64,
    
    /// Minimum required difficulty
    pub min_difficulty: u8,
    
    /// Total hashes submitted this round
    pub total_hashes_submitted: u64,
    
    /// Total rounds completed across all miners
    pub total_rounds_completed: u64,
    
    /// Admin pubkey (can rotate rounds)
    pub admin: Pubkey,
    
    /// PDA bump seed
    pub bump: u8,
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Hash a proof using Keccak256 (ORE-compatible)
/// 
/// Formula: Keccak256(authority || challenge || nonce)
fn hash_proof(authority: &Pubkey, challenge: &[u8; 32], nonce: u64) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(authority.as_ref());
    hasher.update(challenge);
    hasher.update(nonce.to_le_bytes());
    hasher.finalize().into()
}

/// Check if a hash meets the difficulty requirement
/// 
/// Difficulty is measured in leading zero bits.
/// For example, difficulty 8 means the first 8 bits must be zero.
fn check_difficulty(hash: &[u8; 32], difficulty: u8) -> bool {
    let required_zeros = difficulty as usize;
    
    for (i, byte) in hash.iter().enumerate() {
        let leading_zeros = byte.leading_zeros() as usize;
        let bit_pos = i * 8;
        
        if bit_pos + leading_zeros < required_zeros {
            return false;
        }
        
        if leading_zeros < 8 {
            return bit_pos + leading_zeros >= required_zeros;
        }
    }
    
    true
}

/// Generate a new challenge based on clock data
/// 
/// Uses timestamp and slot to create pseudo-random challenge
fn generate_challenge(clock: &Clock) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(clock.unix_timestamp.to_le_bytes());
    hasher.update(clock.slot.to_le_bytes());
    hasher.finalize().into()
}

// ============================================================================
// Error Codes
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("Submitted hash does not meet minimum difficulty requirement")]
    InsufficientDifficulty,
    
    #[msg("Too many submissions - wait at least 1 second between proofs")]
    TooManySubmissions,
    
    #[msg("Difficulty is below the minimum required for this round")]
    DifficultyTooLow,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_difficulty() {
        // All zeros should pass any difficulty
        let easy_hash = [0u8; 32];
        assert!(check_difficulty(&easy_hash, 8));
        assert!(check_difficulty(&easy_hash, 16));

        // Hash starting with 0xFF should fail
        let mut hard_hash = [0u8; 32];
        hard_hash[0] = 0xFF;
        assert!(!check_difficulty(&hard_hash, 1));

        // Hash with one leading zero bit
        let mut medium_hash = [0u8; 32];
        medium_hash[0] = 0b01111111;
        assert!(check_difficulty(&medium_hash, 1));
        assert!(!check_difficulty(&medium_hash, 2));
    }

    #[test]
    fn test_hash_proof() {
        use std::str::FromStr;
        
        let authority = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let challenge = [0u8; 32];
        let nonce = 12345u64;

        let hash = hash_proof(&authority, &challenge, nonce);
        assert_eq!(hash.len(), 32);
    }
}
