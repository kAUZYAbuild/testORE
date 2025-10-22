use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub pubkey: Pubkey,
    pub total_hashes: u64,
    pub rounds_completed: u32,
    pub best_difficulty: u8,
}

/// Fetch and parse the leaderboard from on-chain miner accounts
pub async fn fetch_leaderboard(
    rpc_client: &Arc<RpcClient>,
    program_id: &Pubkey,
    limit: usize,
) -> Result<Vec<LeaderboardEntry>> {
    // Fetch all program accounts (miner PDAs)
    let accounts = rpc_client.get_program_accounts(program_id)?;

    let mut miners: Vec<LeaderboardEntry> = accounts
        .iter()
        .filter_map(|(pubkey, account)| {
            // Parse miner account data
            // Format: [discriminator: 8] [authority: 32] [total_hashes: 8] [rounds: 4] [last_hash: 8] [streak: 4] [best_diff: 1] [bump: 1]
            if account.data.len() >= 66 {
                // Extract authority from account data
                let mut authority_bytes = [0u8; 32];
                authority_bytes.copy_from_slice(&account.data[8..40]);
                let authority = Pubkey::new_from_array(authority_bytes);

                let total_hashes = u64::from_le_bytes(account.data[40..48].try_into().ok()?);
                let rounds_completed = u32::from_le_bytes(account.data[48..52].try_into().ok()?);
                let best_difficulty = account.data.get(64).copied().unwrap_or(0);

                Some(LeaderboardEntry {
                    pubkey: authority,
                    total_hashes,
                    rounds_completed,
                    best_difficulty,
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by total hashes (primary) and rounds completed (secondary)
    miners.sort_by(|a, b| {
        b.total_hashes
            .cmp(&a.total_hashes)
            .then(b.rounds_completed.cmp(&a.rounds_completed))
    });

    Ok(miners.into_iter().take(limit).collect())
}
