use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let actual_data = if data.len() >= 8 { &data[8..] } else { data };

    if actual_data.is_empty() {
        msg!("Err: empty data");
        return Err(ProgramError::InvalidInstructionData);
    }

    let instruction_type = actual_data[0];

    if actual_data.len() < 9 {
        msg!("Err: invalide data: {}", actual_data.len());
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = u64::from_le_bytes([
        actual_data[1],
        actual_data[2],
        actual_data[3],
        actual_data[4],
        actual_data[5],
        actual_data[6],
        actual_data[7],
        actual_data[8],
    ]);
    match instruction_type {
        0 => {
            msg!("running dep");
            deposit(accounts, amount)
        }
        1 => {
            msg!("running withdrawal ");
            withdraw(accounts, amount)
        }
        _ => {
            msg!("Err: invalide: {}", instruction_type);
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

fn deposit(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user = next_account_info(accounts_iter)?;
    let deposit_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    invoke(
        &system_instruction::transfer(user.key, deposit_account.key, amount),
        &[
            user.clone(),
            deposit_account.clone(),
            system_program.clone(),
        ],
    )?;

    Ok(())
}

fn withdraw(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user = next_account_info(accounts_iter)?;
    let deposit_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if deposit_account.lamports() < amount {
        msg!(
            "Insufficient funds: {} need: {}",
            deposit_account.lamports(),
            amount
        );
        return Err(ProgramError::InsufficientFunds);
    }

    invoke(
        &system_instruction::transfer(deposit_account.key, user.key, amount),
        &[
            deposit_account.clone(),
            user.clone(),
            system_program.clone(),
        ],
    )?;

    msg!("Withdrawal completed");
    Ok(())
}
