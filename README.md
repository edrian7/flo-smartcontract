Solana Escrow Program

A simple Solana smart contract that implements a multi‑signature escrow:

Initialize (2‑of‑2): both initializer and taker must sign to create the escrow and lock lamports.

Deposit (1‑of‑2): only the initializer signs to deposit the agreed amount into the escrow account.

Withdraw (2‑of‑2): both parties sign again to release funds from the escrow to the taker.

Table of Contents

Features

Prerequisites

Repository Structure

Building

Deploying

Usage

Accounts

Instructions

Example Client Flow

License

Features

Simple multisig: Uses a seed‑based PDA and Borsh to serialize state.

No Anchor: Pure solana-program and borsh dependencies.

Rent‑exempt escrow: Creates a rent‑exempt PDA account to hold funds.

Prerequisites

Rust 1.70+

Solana CLI

A funded devnet/dev environment or local test validator

Repository Structure

escrow-program/
├── Cargo.toml            # Cargo configuration with solana-program & borsh deps
└── src
    └── lib.rs            # Entrypoint and instruction handlers

Building

cd escrow-program
# Build SBPF-compatible binary for Solana
cargo build-sbf --release

The compiled .so will appear in target/sbf-solana-solana/release/escrow_program.so.

Deploying

Configure your environment:

solana config set --url https://api.devnet.solana.com

Deploy:

solana program deploy \
  target/sbf-solana-solana/release/escrow_program.so

Note the Program ID printed by the CLI; you will use this in your client.

Usage

Accounts

All instructions expect the following account order:

Initializer (signer)

Taker (signer or readonly depending on instruction)

Escrow PDA (writable)

(Optional) System Program

Instructions

Borsh-encoded instruction enum:

pub enum EscrowInstruction {
  Initialize { amount: u64, seed: u8 },
  Deposit {},
  Withdraw {},
}

Initialize (tag = 0)

Keys:

initializer (must be signer)

taker (must be signer)

pda account (writable)

system program

Args: amount (u64 LE), seed (u8)

Creates rent‑exempt PDA and writes EscrowState.

Deposit (tag = 1)

Keys:

initializer (signer + writable)

taker (readonly)

pda account (writable)

system program

Transfers state.amount lamports from initializer into PDA.

Withdraw (tag = 2)

Keys:

initializer (signer)

taker (signer + writable)

pda account (writable)

Moves state.amount lamports from PDA to taker.

Example Client Flow (JavaScript / web3.js)

import {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";

async function initializeEscrow(
  connection, payerKeypair, programId, takerPubkey, amountLamports
) {
  // derive PDA
  const seed = Math.floor(Math.random()*256);
  const [pda, bump] = await PublicKey.findProgramAddress(
    [Buffer.from("escrow"), payerKeypair.publicKey.toBuffer(), Uint8Array.of(seed)],
    programId
  );

  // build instruction data
  const data = Buffer.alloc(10);
  data.writeUInt8(0, 0);
  data.writeBigUInt64LE(BigInt(amountLamports), 1);
  data.writeUInt8(seed, 9);

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payerKeypair.publicKey, isSigner: true, writable: false },
      { pubkey: takerPubkey,            isSigner: true, writable: false },
      { pubkey: pda,                     isSigner: false,writable: true  },
      { pubkey: SystemProgram.programId, isSigner: false,writable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  await sendAndConfirmTransaction(connection, tx, [payerKeypair]);
  return { pda, bump };
}

Replace 0 in data.writeUInt8(0,0) with 1 for Deposit and 2 for Withdraw (and adjust keys).

Contributing

Fork the repo

Create a feature branch

Submit a PR

License

MIT © Flo Protocol

