use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::EarningsWithdrawn;
use crate::state::UserAccount;
use crate::utils;

#[derive(Accounts)]
pub struct WithdrawEarnings<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawEarnings>, amount: u64) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;

    // Airdrop is only unlocked after completing at least 1 gig
    require!(
        user_account.has_completed_gig(),
        TendaError::AirdropStillLocked
    );

    // Check if user has sufficient airdrop balance
    let withdrawable = user_account.withdrawable_balance();
    require!(
        amount <= withdrawable,
        TendaError::InsufficientBalance
    );

    // Deduct from user account
    user_account.deduct_withdrawal(amount)?;
    let remaining_balance = user_account.withdrawable_balance();
    
    // Transfer SOL to user
    **ctx.accounts.user_account.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.user.try_borrow_mut_lamports()? += amount;

    emit!(EarningsWithdrawn {
        user: ctx.accounts.user.key(),
        amount,
        remaining_balance,
        timestamp: utils::current_timestamp()?,
    });

    msg!(
        "User {} withdrew {} lamports, remaining: {}",
        ctx.accounts.user.key(),
        amount,
        remaining_balance
    );

    Ok(())
}
