use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint, entrypoint::ProgramResult,
    msg, program_error::ProgramError,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};
use solana_program::sysvar::Sysvar;

const ESCROW_PDA_SEED: &[u8]  = b"escrow";
const ESCROW_STATE_LEN: usize = 1 + 32 + 32 + 8 + 1;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EscrowState {
    pub is_initialized:    bool,
    pub initializer_pubkey: Pubkey,
    pub taker_pubkey:      Pubkey,
    pub amount:            u64,
    pub bump:              u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum EscrowInstruction {
    Initialize { amount: u64, seed: u8 },
    Deposit {},
    Withdraw {},
}

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instr = EscrowInstruction::try_from_slice(input)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    match instr {
        EscrowInstruction::Initialize { amount, seed } => {
            msg!("Initialize {} lamports, seed {}", amount, seed);
            process_initialize(program_id, accounts, amount, seed)
        }
        EscrowInstruction::Deposit {} => {
            msg!("Deposit");
            process_deposit(accounts)
        }
        EscrowInstruction::Withdraw {} => {
            msg!("Withdraw");
            process_withdraw(accounts)
        }
    }
}

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    seed: u8,
) -> ProgramResult {
    let a               = &mut accounts.iter();
    let initializer     = next_account_info(a)?;
    let taker           = next_account_info(a)?;
    let escrow_account  = next_account_info(a)?;
    let system_program  = next_account_info(a)?;

    if !initializer.is_signer || !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (pda, bump) = Pubkey::find_program_address(
        &[ESCROW_PDA_SEED, initializer.key.as_ref(), &[seed]],
        program_id,
    );
    if pda != *escrow_account.key {
        return Err(ProgramError::InvalidSeeds);
    }
    let rent     = Rent::get()?;
    let lamports = rent.minimum_balance(ESCROW_STATE_LEN);
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            escrow_account.key,
            lamports,
            ESCROW_STATE_LEN as u64,
            program_id,
        ),
        &[initializer.clone(), escrow_account.clone(), system_program.clone()],
        &[&[ESCROW_PDA_SEED, initializer.key.as_ref(), &[seed], &[bump]]],
    )?;

    let state = EscrowState {
        is_initialized:     true,
        initializer_pubkey: *initializer.key,
        taker_pubkey:       *taker.key,
        amount,
        bump,
    };
    state.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?;
    msg!("Escrow initialized at {}", pda);
    Ok(())
}

fn process_deposit(accounts: &[AccountInfo]) -> ProgramResult {
    let a               = &mut accounts.iter();
    let initializer     = next_account_info(a)?;
    let taker           = next_account_info(a)?;
    let escrow_account  = next_account_info(a)?;
    let system_program  = next_account_info(a)?;

    // Only initializer must sign
    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // Verify taker pubkey matches stored state
    let state = EscrowState::try_from_slice(&escrow_account.data.borrow())?;
    if !state.is_initialized || state.taker_pubkey != *taker.key {
        return Err(ProgramError::InvalidAccountData);
    }
    // Transfer amount lamports from initializer → PDA
    invoke(
        &system_instruction::transfer(
            initializer.key,
            escrow_account.key,
            state.amount,
        ),
        &[initializer.clone(), escrow_account.clone(), system_program.clone()],
    )?;
    msg!("Deposited {} lamports", state.amount);
    Ok(())
}

fn process_withdraw(accounts: &[AccountInfo]) -> ProgramResult {
    let a       = &mut accounts.iter();
    let initializer     = next_account_info(a)?;
    let taker           = next_account_info(a)?;
    let escrow_account  = next_account_info(a)?;
    
    if !initializer.is_signer || !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let state = EscrowState::try_from_slice(&escrow_account.data.borrow())?;
    if !state.is_initialized
        || state.initializer_pubkey != *initializer.key
        || state.taker_pubkey != *taker.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut escrow_lamports = escrow_account.lamports.borrow_mut();
    let mut taker_lamports  = taker.lamports.borrow_mut();
    let new_escrow = escrow_lamports
        .checked_sub(state.amount)
        .ok_or(ProgramError::InsufficientFunds)?;
    let new_taker  = taker_lamports
        .checked_add(state.amount)
        .ok_or(ProgramError::InvalidAccountData)?;
    **escrow_lamports = new_escrow;
    **taker_lamports  = new_taker;
    msg!("Withdrew {} lamports", state.amount);
    Ok(())
}