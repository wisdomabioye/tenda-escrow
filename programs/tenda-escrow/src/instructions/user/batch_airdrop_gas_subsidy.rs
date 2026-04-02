use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::constants::*;
use crate::errors::TendaError;
use crate::events::GasSubsidyAirdropped;
use crate::state::UserAccount;
use crate::utils;

#[derive(Accounts)]
pub struct BatchAirdropGasSubsidy {}

/// Process a batch of gas-subsidy airdrops in a single transaction.
///
/// remaining_accounts layout:
///   [treasury, system_program, user_account_pda_0, user_wallet_0, ...]
///
/// The first two slots are always treasury (signer, mut) and system_program.
/// `amounts[i]` corresponds to the pair at [2 + i*2, 2 + i*2 + 1].
/// Entire batch is atomic — any failure reverts all transfers.
///
/// NOTE: The Accounts struct is intentionally empty. Named accounts that include
/// a `Signer<'info>` alongside `remaining_accounts` would trigger an Anchor/Rust
/// lifetime invariance error because `Context` is invariant over `'info`. By
/// placing treasury and system_program inside `remaining_accounts` they all share
/// a single `'info` and the invariance constraint is satisfied.
pub fn handler(
    ctx: Context<BatchAirdropGasSubsidy>,
    amounts: Vec<u64>,
) -> Result<()> {
    let remaining = ctx.remaining_accounts;

    // --------------------------------------------------
    // Batch validation
    // --------------------------------------------------

    require!(!amounts.is_empty(), TendaError::EmptyBatch);

    // Layout must be exactly: [treasury, system_program] + [pda, wallet] × n
    require!(
        remaining.len() == 2 + amounts.len() * 2,
        TendaError::InvalidBatchLength
    );

    // --------------------------------------------------
    // Treasury and System Program validation
    // --------------------------------------------------

    let treasury       = &remaining[0];
    let system_program = &remaining[1];

    require!(treasury.is_signer,   TendaError::TreasuryMustSign);
    require!(treasury.is_writable, TendaError::AccountNotWritable);

    require!(
        system_program.key == &system_program::ID,
        TendaError::InvalidSystemProgram
    );

    // --------------------------------------------------
    // Treasury balance pre-check (prevents mid-loop fail)
    // --------------------------------------------------

    let total_airdrop: u64 = amounts
        .iter()
        .try_fold(0u64, |acc, x| acc.checked_add(*x))
        .ok_or(TendaError::ArithmeticOverflow)?;

    require!(
        treasury.lamports() >= total_airdrop,
        TendaError::InsufficientTreasuryBalance
    );

    let timestamp = utils::current_timestamp()?;

    // --------------------------------------------------
    // Process batch
    // --------------------------------------------------

    for (i, &amount) in amounts.iter().enumerate() {
        require!(
            amount > 0 && amount <= MAX_AIRDROP,
            TendaError::AirdropAmountTooHigh
        );

        let user_account_info = &remaining[2 + i * 2];
        let user_wallet_info  = &remaining[2 + i * 2 + 1];

        // -----------------------------
        // Account safety checks
        // -----------------------------

        require!(
            user_account_info.is_writable,
            TendaError::AccountNotWritable
        );

        require!(
            user_wallet_info.is_writable,
            TendaError::AccountNotWritable
        );

        // user_account_pda must be owned by this program
        require!(
            user_account_info.owner == ctx.program_id,
            TendaError::InvalidUserAccount
        );

        // recipient must be a system-owned account (plain wallet)
        require!(
            user_wallet_info.owner == &system_program::ID,
            TendaError::InvalidRecipient
        );

        // -----------------------------
        // PDA validation
        // -----------------------------

        let (expected_pda, _) = Pubkey::find_program_address(
            &[USER_SEED, user_wallet_info.key.as_ref()],
            ctx.program_id,
        );

        require!(
            expected_pda == *user_account_info.key,
            TendaError::InvalidUserAccount
        );

        // -----------------------------
        // Deserialize user account
        // -----------------------------

        let mut user_account: UserAccount = {
            let data = user_account_info.try_borrow_data()?;
            UserAccount::try_deserialize(&mut &**data)?
        };

        // Guard: only one airdrop allowed per user
        require!(
            !user_account.phone_verified,
            TendaError::AlreadyReceivedAirdrop
        );

        // -----------------------------
        // Transfer SOL
        // -----------------------------

        utils::transfer_sol(
            treasury,
            user_wallet_info,
            amount,
            system_program,
        )?;

        // -----------------------------
        // Update state
        // -----------------------------

        user_account.airdrop_sol = user_account
            .airdrop_sol
            .checked_add(amount)
            .ok_or(TendaError::ArithmeticOverflow)?;

        user_account.phone_verified = true;

        {
            let mut data = user_account_info.try_borrow_mut_data()?;
            user_account.try_serialize(&mut &mut **data)?;
        }

        // -----------------------------
        // Emit event
        // -----------------------------

        emit!(GasSubsidyAirdropped {
            user: user_wallet_info.key(),
            amount,
            timestamp,
        });

        msg!(
            "Batch airdrop [{}/{}]: {} lamports → {}",
            i + 1,
            amounts.len(),
            amount,
            user_wallet_info.key()
        );
    }

    Ok(())
}
