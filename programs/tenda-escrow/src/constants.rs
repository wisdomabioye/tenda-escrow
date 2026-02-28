use anchor_lang::prelude::*;

// ==================== SEEDS ====================

#[constant]
pub const PLATFORM_SEED: &[u8] = b"platform";

#[constant]
pub const USER_SEED: &[u8] = b"user";

#[constant]
pub const ESCROW_SEED: &[u8] = b"escrow";

// ==================== LIMITS ====================

/// Minimum gig payment (0.001 SOL = 1,000,000 lamports)
pub const MIN_PAYMENT: u64 = 1_000_000;

/// Maximum platform fee (5% = 500 basis points)
pub const MAX_PLATFORM_FEE_BPS: u16 = 500;

/// Maximum airdrop amount (0.01 SOL = 10,000,000 lamports)
pub const MAX_AIRDROP: u64 = 10_000_000;

/// Default gas subsidy (0.005 SOL = 5,000,000 lamports)
pub const DEFAULT_GAS_SUBSIDY: u64 = 5_000_000;

/// Default grace period (24 hours = 86400 seconds)
pub const DEFAULT_GRACE_PERIOD: i64 = 86_400;

/// Maximum gig ID length (UUID without hyphens — 32 hex chars = 32 bytes, fits Solana seed limit)
pub const MAX_GIG_ID_LEN: usize = 32;

/// Minimum completion duration: 1 hour in seconds
pub const MIN_COMPLETION_DURATION_SECONDS: u64 = 3_600;

/// Maximum completion duration: 90 days in seconds
pub const MAX_COMPLETION_DURATION_SECONDS: u64 = 90 * 24 * 3_600;

/// Maximum dispute reason length
pub const MAX_DISPUTE_REASON_LEN: usize = 1000;

// ==================== ACCOUNT SIZES ====================

/// PlatformState account size
/// 8 (discriminator) + 32 (admin) + 2 (fee_bps) + 32 (treasury) + 8 (total_gigs) + 8 (total_volume) + 8 (grace_period)
pub const PLATFORM_STATE_SIZE: usize = 8 + 32 + 2 + 32 + 8 + 8 + 8;

/// UserAccount size
/// 8 (discriminator) + 32 (wallet) + 8 (airdrop_sol) + 8 (earned_sol) + 4 (completed_gigs) + 1 (phone_verified) + 8 (created_at)
pub const USER_ACCOUNT_SIZE: usize = 8 + 32 + 8 + 8 + 4 + 1 + 8;

