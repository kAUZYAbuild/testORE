TestORE Quick Start Guide
Get mining in 5 minutes! ⚡
Step 1: Install Prerequisites
Rust
bashcurl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
Solana CLI
bashsh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
Verify
bashrustc --version  # Should be 1.75+
solana --version # Should be 1.18+
Step 2: Build TestORE
bashcd testore
cargo build --release
⏱️ First build takes 5-10 minutes.
Step 3: Setup Wallet
bash# Create new wallet
solana-keygen new --outfile ~/.config/solana/id.json

# Or if you already have a wallet, make sure it's at the default location
# or use --keypair flag when running commands
Step 4: Get Testnet SOL
bash# Method 1: Using TestORE CLI
./target/release/testore airdrop 1.0

# Method 2: Using Solana CLI
solana airdrop 1 --url testnet

# Verify balance
./target/release/testore balance
Step 5: Initialize Miner
bash./target/release/testore init
You should see:
✅ Miner initialized!
   PDA: [YOUR_MINER_ADDRESS]
   
   Ready to mine! Run: testore mine
Step 6: Start Mining!
bash# Basic mining (all CPU cores)
./target/release/testore mine

# Or with options
./target/release/testore mine --threads 4 --difficulty 8

# For 24/7 mining (recommended)
./target/release/testore mine --forever
📊 Check Progress
bash# View leaderboard
./target/release/testore leaderboard

# Check your stats
./target/release/testore stats

# Check balance
./target/release/testore balance
🏆 Track Your Airdrop Eligibility
You need 100,000 hashes minimum to qualify.
Progress Tracker:

100K hashes = 10 TESTORE ✅
1M hashes = 100 TESTORE 🎯
10M hashes = 1,000 TESTORE 🚀
100M hashes = 10,000 TESTORE 💰

🔧 Troubleshooting
"Keypair file not found"
bashsolana-keygen new --outfile ~/.config/solana/id.json
"Insufficient funds"
bash./target/release/testore airdrop 1.0
"Connection refused"

Check internet connection
Verify testnet is up: https://status.solana.com

Mining too slow

Close unnecessary programs
Verify thread count: --threads $(nproc)
Check CPU isn't thermal throttling

💡 Pro Tips
1. Run in screen/tmux for 24/7
bashscreen -S testore
./target/release/testore mine --forever
# Detach: Ctrl+A, then D
# Reattach: screen -r testore
2. Monitor with logs
bashRUST_LOG=info ./target/release/testore mine 2>&1 | tee mining.log
3. Auto-check leaderboard
bashwatch -n 60 './target/release/testore leaderboard --top 5'
4. Optimize performance
bash# Use all cores
./target/release/testore mine --threads $(nproc)

# Higher difficulty = more valuable proofs
./target/release/testore mine --difficulty 10
🐧 Linux/Mac Specific
Add to PATH
bash# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/testore/target/release:$PATH"

# Now you can just run:
testore mine
Create systemd service (Linux)
bashsudo nano /etc/systemd/system/testore.service
Paste:
ini[Unit]
Description=TestORE Miner
After=network.target

[Service]
Type=simple
User=YOUR_USERNAME
WorkingDirectory=/home/YOUR_USERNAME/testore
ExecStart=/home/YOUR_USERNAME/testore/target/release/testore mine --forever
Restart=always

[Install]
WantedBy=multi-user.target
Enable:
bashsudo systemctl enable testore
sudo systemctl start testore
sudo systemctl status testore
🪟 Windows Specific
Using PowerShell
powershell# Build
cargo build --release

# Run
.\target\release\testore.exe mine

# Forever mode
.\target\release\testore.exe mine --forever
Double-click batch file
Create start-mining.bat:
batch@echo off
cd /d %~dp0
target\release\testore.exe mine --forever
pause
🌐 Browser Mining (Optional)
For casual mining, open web/index.html:
bashcd web
python3 -m http.server 8000
# Open http://localhost:8000
⚠️ Browser mining is ~100x slower than CLI!
✅ Verification Checklist

 Rust 1.75+ installed
 Solana CLI 1.18+ installed
 TestORE built successfully
 Wallet created/imported
 Testnet SOL acquired (>0.1)
 Miner initialized
 Mining started successfully
 Can see stats/leaderboard

📚 Next Steps
Once mining:

Monitor progress: Check testore stats regularly
Join community: Discord, Twitter
Track airdrops: Monthly snapshots
Optimize: Experiment with threads/difficulty
Contribute: Report bugs, suggest features

🆘 Still Having Issues?

Read full docs: See README.md
Check logs: Look for error messages
Ask community: Discord or GitHub Issues
Provide details: OS, versions, error output

