use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};
use solana_program::program::invoke;
use solana_program::sysvar::Sysvar;

const ESCROW_PDA_SEED: &[u8]     = b"escrow";
const ESCROW_STATE_LEN: usize    = 1 + 32 + 32 + 8 + 1;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EscrowState {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub taker_pubkey: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum EscrowInstruction {
    Initialize { amount: u64 },
    Deposit {},
    Withdraw {},
}

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = EscrowInstruction::try_from_slice(input)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    match instruction {
        EscrowInstruction::Initialize { amount } => {
            msg!("Escrow: Initialize with {}", amount);
            process_initialize(program_id, accounts, amount)
        }
        EscrowInstruction::Deposit {} => {
            msg!("Escrow: Deposit");
            process_deposit(program_id, accounts)
        }
        EscrowInstruction::Withdraw {} => {
            msg!("Escrow: Withdraw");
            process_withdraw(program_id, accounts)
        }
    }
}

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let a = &mut accounts.iter();
    let initializer     = next_account_info(a)?;
    let taker           = next_account_info(a)?;
    let escrow_account  = next_account_info(a)?;
    let system_program  = next_account_info(a)?;

    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, bump) = Pubkey::find_program_address(
        &[ESCROW_PDA_SEED, initializer.key.as_ref()],
        program_id,
    );
    if pda != *escrow_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent         = Rent::get()?;
    let lamports_req = rent.minimum_balance(ESCROW_STATE_LEN);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            escrow_account.key,
            lamports_req,
            ESCROW_STATE_LEN as u64,
            program_id,
        ),
        &[initializer.clone(), escrow_account.clone(), system_program.clone()],
        &[&[ESCROW_PDA_SEED, initializer.key.as_ref(), &[bump]]],
    )?;

    let escrow_state = EscrowState {
        is_initialized:    true,
        initializer_pubkey: *initializer.key,
        taker_pubkey:      *taker.key,
        amount,
        bump,
    };
    escrow_state.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?;
    msg!("Initialized escrow—PDA: {}, amount: {}", pda, amount);

    Ok(())
}

fn process_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initializer     = next_account_info(account_info_iter)?;
    let taker           = next_account_info(account_info_iter)?;
    let escrow_account  = next_account_info(account_info_iter)?;
    let system_program  = next_account_info(account_info_iter)?;

    if !initializer.is_signer || !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut escrow_state = EscrowState::try_from_slice(&escrow_account.data.borrow())?;
    if !escrow_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    if escrow_state.initializer_pubkey != *initializer.key
        || escrow_state.taker_pubkey != *taker.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let transfer_ix = system_instruction::transfer(
        initializer.key,
        escrow_account.key,
        escrow_state.amount,
    );
    invoke(
        &transfer_ix,
        &[
            initializer.clone(),
            escrow_account.clone(),
            system_program.clone(),
        ],
    )?;

    msg!(
        "Deposited {} lamports into escrow {}",
        escrow_state.amount,
        escrow_account.key
    );
    Ok(())
}

fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let a = &mut accounts.iter();
    let initializer    = next_account_info(a)?;
    let taker          = next_account_info(a)?;
    let escrow_account = next_account_info(a)?;

    if !initializer.is_signer || !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let escrow_state = EscrowState::try_from_slice(&escrow_account.data.borrow())?;
    if !escrow_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    if escrow_state.initializer_pubkey != *initializer.key
        || escrow_state.taker_pubkey      != *taker.key
    {
        return Err(ProgramError::InvalidAccountData);
    }

    let (pda, bump) = Pubkey::find_program_address(
        &[ESCROW_PDA_SEED, initializer.key.as_ref()],
        program_id,
    );
    if pda != *escrow_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    {
        let mut escrow_lamports = escrow_account
            .lamports
            .borrow_mut();
        let mut taker_lamports = taker
            .lamports
            .borrow_mut();

        let new_escrow = escrow_lamports
            .checked_sub(escrow_state.amount)
            .ok_or(ProgramError::InsufficientFunds)?;
        let new_taker = taker_lamports
            .checked_add(escrow_state.amount)
            .ok_or(ProgramError::InvalidAccountData)?;

        **escrow_lamports = new_escrow;
        **taker_lamports  = new_taker;
    }

    msg!(
        "Withdrew {} lamports from escrow {} to {}",
        escrow_state.amount,
        escrow_account.key,
        taker.key
    );
    Ok(())
}



