use anchor_lang::prelude::*;

#[error_code]
pub enum TendaError {
    // ==================== PLATFORM ERRORS ====================
    
    #[msg("Platform fee exceeds maximum allowed (5%)")]
    PlatformFeeTooHigh,

    #[msg("Seeker fee must not exceed the standard platform fee")]
    SeekerFeeExceedsStandardFee,

    #[msg("Platform is already initialized")]
    PlatformAlreadyInitialized,

    // ==================== USER ERRORS ====================
    
    #[msg("User account already exists")]
    UserAccountAlreadyExists,

    #[msg("User has already received gas subsidy")]
    AlreadyReceivedAirdrop,

    #[msg("Airdrop amount exceeds maximum allowed")]
    AirdropAmountTooHigh,

    #[msg("Insufficient balance to withdraw")]
    InsufficientBalance,

    #[msg("Must complete at least 1 gig to unlock airdrop")]
    AirdropStillLocked,

    #[msg("User account does not exist")]
    UserAccountNotFound,

    #[msg("Batch must contain at least one recipient")]
    EmptyBatch,

    #[msg("Remaining accounts length must equal 2 × number of amounts")]
    InvalidBatchLength,

    #[msg("User account PDA does not match expected derivation for this wallet")]
    InvalidUserAccount,

    #[msg("Treasury account must be a signer")]
    TreasuryMustSign,

    #[msg("Account must be writable")]
    AccountNotWritable,

    #[msg("System program account is invalid")]
    InvalidSystemProgram,

    #[msg("Treasury has insufficient balance for this batch")]
    InsufficientTreasuryBalance,

    #[msg("Recipient wallet must be a system-owned account")]
    InvalidRecipient,

    // ==================== ESCROW ERRORS ====================
    
    #[msg("Payment amount below minimum")]
    PaymentTooLow,

    #[msg("Deadline must be in the future")]
    InvalidDeadline,

    #[msg("Accept deadline has passed")]
    AcceptDeadlinePassed,

    #[msg("Completion duration is below minimum allowed")]
    DurationTooShort,

    #[msg("Completion duration exceeds maximum allowed")]
    DurationTooLong,

    #[msg("Gig ID is too long")]
    GigIdTooLong,

    #[msg("Insufficient funds for escrow deposit")]
    InsufficientFunds,

    #[msg("Invalid gig status for this operation")]
    InvalidGigStatus,

    #[msg("Caller is not the poster")]
    NotPoster,

    #[msg("Caller is not the worker")]
    NotWorker,

    #[msg("Cannot accept own gig")]
    CannotAcceptOwnGig,

    #[msg("Gig is not open for acceptance")]
    GigNotOpen,

    #[msg("Gig has not been accepted yet")]
    GigNotAccepted,

    #[msg("Proof has not been submitted")]
    ProofNotSubmitted,

    #[msg("Gig has not expired yet")]
    GigNotExpired,

    #[msg("Cannot refund gig with submitted proof")]
    CannotRefundWithProof,

    #[msg("Submission deadline has passed")]
    SubmissionDeadlinePassed,

    // ==================== DISPUTE ERRORS ====================
    
    #[msg("Dispute reason is too long")]
    DisputeReasonTooLong,

    #[msg("Cannot dispute gig in current status")]
    CannotDispute,

    #[msg("Caller is not authorized to dispute")]
    NotAuthorizedToDispute,

    #[msg("Gig is not disputed")]
    GigNotDisputed,

    #[msg("Caller is not admin")]
    NotAdmin,

    // ==================== ARITHMETIC ERRORS ====================
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,
}
