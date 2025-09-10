use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VoteRecord {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub vote_weight: u64,
    pub vote_type: VoteType,
    pub voted_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum VoteType {
    Yes,
    No,
    Abstain,
}
