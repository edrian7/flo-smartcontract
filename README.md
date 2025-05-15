# Solana Escrow Program

A simple Solana smart contract that implements a multi-signature escrow:

- **Initialize** : both Sender and Receiver must sign to create the escrow and lock lamports.
- **Deposit** : only the Sender signs to deposit the agreed amount into the escrow account.
- **Withdraw** : both parties sign again to release funds from the escrow to the Receiver.

---

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Repository Structure](#repository-structure)
- [Building](#building)
- [Deploying](#deploying)

---

## Features

- **Simple multisig**: Uses a seed-based PDA and Borsh to serialize state.
- **No Anchor**: Pure `solana-program` and `borsh` dependencies.
- **Rent-exempt escrow**: Creates a rent-exempt PDA account to hold funds.

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- A funded devnet/dev environment or local test validator

---

## Repository Structure

```text
escrow-program/
├── Cargo.toml            # Cargo configuration with solana-program & borsh deps
└── src/
    └── lib.rs            # Entrypoint and instruction handlers
```

---

## Building

```
cd escrow-program
# Build SBPF-compatible binary for Solana
cargo build-sbf --release
```

## Deploying

```
# 1. Configure your Solana CLI to point at devnet
solana config set --url https://api.devnet.solana.com

# 2. Deploy the compiled program
solana program deploy \
  target/sbf-solana-solana/release/escrow_program.so

# 3. Note the Program ID printed by the CLI; you will use this in your client.
```


