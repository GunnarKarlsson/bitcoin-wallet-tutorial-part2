# Bitcoin Wallet in Rust — Part II

Companion code for part II of the [Let's Build a Bitcoin Wallet in Rust](https://medium.com/@gunnar.h.karlsson/lets-build-a-bitcoin-wallet-in-rust-part-ii-c218a4dcb96d) tutorial series.

This part connects the wallet from part I to a local Bitcoin Core testnet node over RPC, imports the address as a watch-only descriptor, and sums UTXOs to show your balance.

Requires Rust 1.85 or later and a Bitcoin Core testnet node listening on `http://127.0.0.1:18332`. Update the RPC username and password in `get_rpc_client()` to match your node.

## Run

Create a wallet (if you do not already have `wallet.json`):

```bash
cargo run -- new
```

Print the receive address (paste it into a testnet faucet):

```bash
cargo run -- receive
```

Show the confirmed testnet balance:

```bash
cargo run -- balance
```

This is testnet learning code. The private key is stored unencrypted in `wallet.json` — do not use it on mainnet.

## License

MIT. See [LICENSE](LICENSE).
