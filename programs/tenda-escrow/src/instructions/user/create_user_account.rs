use anchor_lang::prelude::*;
use crate::constants::*;
use crate::events::UserAccountCreated;
use crate::state::UserAccount;
use crate::utils;

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(
        init,
        payer = user,
        space = UserAccount::LEN,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateUserAccount>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;

    user_account.wallet = ctx.accounts.user.key();
    user_account.airdrop_sol = 0;
    user_account.earned_sol = 0;
    user_account.completed_gigs = 0;
    user_account.phone_verified = false;
    user_account.created_at = utils::current_timestamp()?;

    emit!(UserAccountCreated {
        wallet: ctx.accounts.user.key(),
        timestamp: user_account.created_at,
    });

    msg!("User account created for: {}", ctx.accounts.user.key());

    Ok(())
}
