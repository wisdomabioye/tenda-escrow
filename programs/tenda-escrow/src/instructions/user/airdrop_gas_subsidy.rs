use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::GasSubsidyAirdropped;
use crate::state::UserAccount;
use crate::utils;

#[derive(Accounts)]
pub struct AirdropGasSubsidy<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: User receiving the airdrop
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    /// Platform treasury that sends the airdrop
    #[account(mut)]
    pub treasury: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AirdropGasSubsidy>, amount: u64) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;

    // Check if already received airdrop
    require!(
        !user_account.phone_verified,
        TendaError::AlreadyReceivedAirdrop
    );

    // Check airdrop amount
    require!(
        amount <= MAX_AIRDROP,
        TendaError::AirdropAmountTooHigh
    );

    // Transfer SOL from treasury to user
    utils::transfer_sol(
        &ctx.accounts.treasury.to_account_info(),
        &ctx.accounts.user.to_account_info(),
        amount,
        &ctx.accounts.system_program.to_account_info(),
    )?;

    // Update user account
    user_account.airdrop_sol = user_account.airdrop_sol
        .checked_add(amount)
        .ok_or(TendaError::ArithmeticOverflow)?;
    user_account.phone_verified = true;

    emit!(GasSubsidyAirdropped {
        user: ctx.accounts.user.key(),
        amount,
        timestamp: utils::current_timestamp()?,
    });

    msg!("Airdropped {} lamports to user: {}", amount, ctx.accounts.user.key());

    Ok(())
}
