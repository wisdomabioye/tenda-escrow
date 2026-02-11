use anchor_lang::prelude::*;
use crate::errors::TendaError;

/// Calculate platform fee from payment amount
pub fn calculate_platform_fee(payment_amount: u64, fee_bps: u16) -> Result<u64> {
    let fee = (payment_amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(TendaError::ArithmeticOverflow)?
        .checked_div(10000)
        .ok_or(TendaError::ArithmeticOverflow)?;
    
    Ok(fee as u64)
}

/// Get current timestamp
pub fn current_timestamp() -> Result<i64> {
    Clock::get()?.unix_timestamp.try_into()
        .map_err(|_| error!(TendaError::ArithmeticOverflow))
}

/// Transfer SOL from one account to another
pub fn transfer_sol<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    amount: u64,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        from.key,
        to.key,
        amount,
    );

    anchor_lang::solana_program::program::invoke(
        &ix,
        &[from.clone(), to.clone(), system_program.clone()],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_platform_fee() {
        // 2% of 1 SOL (1_000_000_000 lamports)
        let fee = calculate_platform_fee(1_000_000_000, 200).unwrap();
        assert_eq!(fee, 20_000_000); // 0.02 SOL

        // 5% of 0.5 SOL (500_000_000 lamports)
        let fee = calculate_platform_fee(500_000_000, 500).unwrap();
        assert_eq!(fee, 25_000_000); // 0.025 SOL
    }
}
