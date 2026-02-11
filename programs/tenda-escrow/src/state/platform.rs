use anchor_lang::prelude::*;

#[account]
pub struct PlatformState {
    /// Platform admin wallet
    pub admin: Pubkey,
    
    /// Platform fee in basis points (e.g., 200 = 2%)
    pub platform_fee_bps: u16,
    
    /// Platform treasury for collecting fees
    pub treasury: Pubkey,
    
    /// Total number of gigs created
    pub total_gigs: u64,
    
    /// Total volume processed in lamports
    pub total_volume: u64,
    
    /// Grace period after deadline in seconds (default: 86400 = 24h)
    pub grace_period_seconds: i64,
}

impl PlatformState {
    pub const LEN: usize = 8 + // discriminator
        32 +  // admin
        2 +   // platform_fee_bps
        32 +  // treasury
        8 +   // total_gigs
        8 +   // total_volume
        8;    // grace_period_seconds
}
