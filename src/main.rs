use anyhow::{Context, Result};
use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Network, PrivateKey, PublicKey};
use bitcoincore_rpc::bitcoin::Address as BtcAddress;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::json::{ImportDescriptors, ImportMultiResult, Timestamp};
use secp256k1::{Secp256k1, rand::rngs::OsRng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct Wallet {
    private_key: String, // WIF format
    address: String,
}

fn wallet_path() -> Result<PathBuf, std::io::Error> {
    std::env::current_dir().map(|mut path| {
        path.push("wallet.json");
        path
    })
}

fn save_wallet(wallet: &Wallet) -> Result<()> {
    let path = wallet_path()?;
    let json = serde_json::to_string_pretty(wallet)?;
    fs::write(&path, json)?;
    println!("Wallet saved to: {}", path.display());
    Ok(())
}

fn load_wallet() -> Result<Wallet> {
    let path = wallet_path()?;
    let json = fs::read_to_string(&path).context("No wallet found. Run 'new' to create one.")?;
    let wallet: Wallet = serde_json::from_str(&json)?;
    Ok(wallet)
}

fn generate_new_wallet() -> Result<Wallet> {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    let privkey = PrivateKey::new(secret_key, Network::Testnet);
    let address = Address::p2pkh(PublicKey::new(public_key), Network::Testnet);

    let wallet = Wallet {
        private_key: privkey.to_wif(),
        address: address.to_string(),
    };

    println!("New wallet created!");
    println!("Address (send testnet BTC here): {}", wallet.address);
    println!("Private Key (WIF) — KEEP SECRET!: {}", wallet.private_key);

    save_wallet(&wallet)?;
    Ok(wallet)
}

fn get_rpc_client() -> Result<Client> {
    let rpc_url = "http://127.0.0.1:18332"; // your local testnet node
    let auth = Auth::UserPass("admin".to_string(), "abc123".to_string());
    Client::new(rpc_url, auth).context("Failed to connect to Bitcoin Core RPC")
}

fn ensure_descriptor_imported(client: &Client, address: &str) -> Result<()> {
    
    let plain_desc = format!("addr({})", address);

    // Step 1: Get checksummed descriptor via getdescriptorinfo
    let info = client.get_descriptor_info(&plain_desc)
        .context("getdescriptorinfo failed (check if address matches network: testnet addresses start with m/n/tb1)")?;

    let checksummed_desc = info.descriptor;

    let import_req = ImportDescriptors {
        // This is the address the node should watch.  
        // We added a little security code (checksum) at the end so the node knows it's not typed wrong.
        descriptor: checksummed_desc,    
        // Only start looking for BTC sent to this address from right now onward.             
        timestamp: Timestamp::Now,  
        // Don't use this address to automatically create new addresses.              
        active: Some(false),                     
        // There's no range here. This is just one single, fixed address. 
        range: None,                  
        // Since we're not generating new addresses automatically (active is false),  
        // there's no 'next number in line' to keep track of.             
        next_index: None,                
        // This isn't a 'change' address         
        internal: None,
        // Give this address a user-friendly name
        label: Some("rust-wallet".to_string()),
    };

    let results: Vec<ImportMultiResult> = client.import_descriptors(import_req)?;

    for res in results {
        if let Some(err) = res.error {
            let msg = err.message.to_lowercase();
            if !msg.contains("already") && !msg.contains("exists") {
                return Err(anyhow::anyhow!("import_descriptors failed: {:?}", err));
            }
        } else if res.success {
            println!("Descriptor imported successfully (watch-only via combo).");
        }
    }

    Ok(())
}

fn get_balance(client: &Client, address: &str) -> Result<f64> {
    ensure_descriptor_imported(client, address)?;

    let addr: BtcAddress = address
        .parse::<Address<NetworkUnchecked>>()
        .context("Invalid address")?
        .assume_checked();
    let utxos = client.list_unspent(
        Some(1), // min confirmations. ≥ 1 confirmation (excludes mempool / 0-conf)
        None,    // max confirmations
        Some(&[&addr]),
        Some(false),
        None,
    )?;

    let total: f64 = utxos.iter().map(|utxo| utxo.amount.to_btc()).sum();
    Ok(total)
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "btc-wallet", about = "Minimal Bitcoin testnet wallet in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new wallet
    New,
    /// Show current balance
    Balance,
    /// Show receive address
    Receive,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New) => {
            generate_new_wallet()?;
        }
        Some(Commands::Balance) => {
            let wallet = load_wallet()?;
            let client = get_rpc_client()?;
            let balance = get_balance(&client, &wallet.address)?;
            println!("Balance: {:.8} tBTC", balance);
        }
        Some(Commands::Receive) | None => {
            let wallet = load_wallet()?;
            println!("Receive address: {}", wallet.address);
            println!("Tip: Paste this into a testnet faucet!");
        }
    }
    Ok(())
}
