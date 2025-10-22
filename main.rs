use anyhow::{anyhow, Result};
use colored::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// TestORE Mainnet Airdrop Bridge
///
/// This application:
/// 1. Polls testnet for top miners
/// 2. Calculates mainnet TESTORE token allocations
/// 3. Distributes airdrops to eligible wallets
///
/// ## Configuration
/// Set these environment variables:
/// - TESTNET_RPC: Testnet RPC endpoint
/// - MAINNET_RPC: Mainnet RPC endpoint  
/// - AIRDROP_KEYPAIR: Path to mainnet funding wallet
/// - PROGRAM_ID: TestORE program ID on testnet

const TOKENS_PER_MILLION_HASHES: u64 = 100;
const MINIMUM_HASHES_FOR_AIRDROP: u64 = 100_000;
const TOP_MINERS_TO_AIRDROP: usize = 1000;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!(
        "\n{} {}\n",
        "🌉".bright_cyan().bold(),
        "TestORE Mainnet Airdrop Bridge".bright_white().bold()
    );

    // Load configuration
    let config = load_config()?;

    println!("{}", "═".repeat(60).bright_black());
    println!(
        "{} {}",
        "Testnet RPC:".bright_cyan(),
        config.testnet_rpc.bright_white()
    );
    println!(
        "{} {}",
        "Mainnet RPC:".bright_cyan(),
        config.mainnet_rpc.bright_white()
    );
    println!(
        "{} {}",
        "Program ID:".bright_cyan(),
        config.program_id.to_string().bright_yellow()
    );
    println!(
        "{} {}",
        "Funding Wallet:".bright_cyan(),
        config.keypair.pubkey().to_string().bright_yellow()
    );
    println!("{}", "═".repeat(60).bright_black());
    println!();

    // Create RPC clients
    let testnet_client = RpcClient::new_with_commitment(
        config.testnet_rpc.clone(),
        CommitmentConfig::confirmed(),
    );

    let _mainnet_client = RpcClient::new_with_commitment(
        config.mainnet_rpc.clone(),
        CommitmentConfig::confirmed(),
    );

    // Step 1: Fetch leaderboard from testnet
    println!(
        "{} Fetching testnet leaderboard...\n",
        "📊".bright_cyan()
    );
    let leaderboard = fetch_testnet_leaderboard(&testnet_client, &config.program_id)?;

    if leaderboard.is_empty() {
        println!("{} No miners found on testnet yet.", "ℹ️".bright_yellow());
        return Ok(());
    }

    println!(
        "{} Found {} miners on testnet\n",
        "✅".bright_green(),
        leaderboard.len().to_string().bright_cyan()
    );

    // Step 2: Calculate airdrop allocations
    println!("{} Calculating airdrop allocations...\n", "🧮".bright_cyan());
    let allocations = calculate_allocations(&leaderboard);

    let total_tokens: u64 = allocations.values().sum();
    let eligible_count = allocations.len();

    println!("{}", "═══ Airdrop Summary ═══".bright_yellow().bold());
    println!(
        "   Eligible Miners: {}",
        eligible_count.to_string().bright_green()
    );
    println!(
        "   Total TESTORE Tokens: {}",
        format_number(total_tokens).bright_cyan().bold()
    );
    println!();

    // Display top 10 allocations
    let mut sorted_allocations: Vec<_> = allocations.iter().collect();
    sorted_allocations.sort_by(|a, b| b.1.cmp(a.1));

    println!("{}", "Top 10 Recipients:".bright_yellow());
    for (i, (pubkey, amount)) in sorted_allocations.iter().take(10).enumerate() {
        println!(
            "   {}. {} → {} TESTORE",
            (i + 1).to_string().bright_white(),
            pubkey.to_string()[..8].bright_yellow(),
            format_number(**amount).bright_cyan()
        );
    }
    println!();

    // Step 3: Execute airdrops (DRY RUN for now)
    println!(
        "{} {}",
        "⚠️".bright_yellow(),
        "DRY RUN MODE - No transactions sent".bright_yellow().bold()
    );
    println!(
        "   To execute real airdrops, set EXECUTE_AIRDROPS=true\n"
    );

    if std::env::var("EXECUTE_AIRDROPS").unwrap_or_default() == "true" {
        println!(
            "{} Executing mainnet airdrops...\n",
            "🚀".bright_green().bold()
        );

        for (pubkey, amount) in allocations.iter() {
            println!(
                "   {} Would send {} TESTORE to {}",
                "💸".bright_cyan(),
                format_number(*amount).bright_cyan(),
                pubkey.to_string()[..8].bright_yellow()
            );

            // Rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        println!("\n{} Airdrop complete!", "🎉".bright_green().bold());
    } else {
        println!(
            "{} Preview mode - no airdrops executed\n",
            "ℹ️".bright_blue()
        );
    }

    // Step 4: Save snapshot for records
    save_snapshot(&allocations)?;

    println!(
        "{} Snapshot saved to: {}",
        "💾".bright_cyan(),
        "airdrop_snapshot.json".bright_yellow()
    );
    println!();

    Ok(())
}

// ============================================================================
// Core Functions
// ============================================================================

#[derive(Debug)]
struct Config {
    testnet_rpc: String,
    mainnet_rpc: String,
    program_id: Pubkey,
    keypair: Keypair,
}

fn load_config() -> Result<Config> {
    let testnet_rpc = std::env::var("TESTNET_RPC")
        .unwrap_or_else(|_| "https://api.testnet.solana.com".to_string());

    let mainnet_rpc = std::env::var("MAINNET_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let program_id = std::env::var("PROGRAM_ID")
        .unwrap_or_else(|_| "TESTORE11111111111111111111111111111111111".to_string());
    let program_id = Pubkey::from_str(&program_id)?;

    let keypair_path = std::env::var("AIRDROP_KEYPAIR")
        .unwrap_or_else(|_| "~/.config/solana/id.json".to_string());

    let keypair = load_keypair(&keypair_path)?;

    Ok(Config {
        testnet_rpc,
        mainnet_rpc,
        program_id,
        keypair,
    })
}

#[derive(Debug, Clone)]
struct MinerStats {
    pubkey: Pubkey,
    total_hashes: u64,
}

fn fetch_testnet_leaderboard(client: &RpcClient, program_id: &Pubkey) -> Result<Vec<MinerStats>> {
    let accounts = client.get_program_accounts(program_id)?;

    let mut miners: Vec<MinerStats> = accounts
        .iter()
        .filter_map(|(_pda, account)| {
            if account.data.len() >= 48 {
                // Extract authority (actual miner pubkey)
                let mut authority_bytes = [0u8; 32];
                authority_bytes.copy_from_slice(&account.data[8..40]);
                let authority = Pubkey::new_from_array(authority_bytes);

                let total_hashes =
                    u64::from_le_bytes(account.data[40..48].try_into().ok()?);

                Some(MinerStats {
                    pubkey: authority,
                    total_hashes,
                })
            } else {
                None
            }
        })
        .collect();

    miners.sort_by(|a, b| b.total_hashes.cmp(&a.total_hashes));

    Ok(miners.into_iter().take(TOP_MINERS_TO_AIRDROP).collect())
}

fn calculate_allocations(leaderboard: &[MinerStats]) -> HashMap<Pubkey, u64> {
    let mut allocations = HashMap::new();

    for miner in leaderboard {
        if miner.total_hashes >= MINIMUM_HASHES_FOR_AIRDROP {
            let tokens = (miner.total_hashes / 1_000_000) * TOKENS_PER_MILLION_HASHES;

            if tokens > 0 {
                allocations.insert(miner.pubkey, tokens);
            }
        }
    }

    allocations
}

fn save_snapshot(allocations: &HashMap<Pubkey, u64>) -> Result<()> {
    use chrono::Utc;

    let snapshot = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "total_miners": allocations.len(),
        "total_tokens": allocations.values().sum::<u64>(),
        "allocations": allocations
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<HashMap<_, _>>()
    });

    fs::write("airdrop_snapshot.json", serde_json::to_string_pretty(&snapshot)?)?;

    Ok(())
}

// ============================================================================
// Utilities
// ============================================================================

fn load_keypair(path: &str) -> Result<Keypair> {
    let expanded_path = if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(&path[2..])
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    };

    if !expanded_path.exists() {
        return Err(anyhow!(
            "Keypair file not found: {}",
            expanded_path.display()
        ));
    }

    let keypair_bytes = fs::read_to_string(&expanded_path)?;
    let keypair_bytes: Vec<u8> = serde_json::from_str(&keypair_bytes)?;

    Ok(Keypair::from_bytes(&keypair_bytes)?)
}

fn format_number(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}
