use anchor_lang::prelude::*;

#[constant]
pub const GOVERNANCE_REALM_SEED: &str = "governance_realm";

#[constant]
pub const PROPOSAL_SEED: &str = "proposal";

#[constant]
pub const VOTE_RECORD_SEED: &str = "vote_record";

// Time constants (in seconds)
pub const MIN_VOTING_TIME: u32 = 3600; // 1 hour minimum
pub const MAX_VOTING_TIME: u32 = 604800; // 1 week maximum

// Voting thresholds
pub const MIN_YES_VOTE_THRESHOLD: u8 = 1; // At least 1% yes votes
pub const MAX_YES_VOTE_THRESHOLD: u8 = 100; // Max 100% yes votes
