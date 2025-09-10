use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GovernanceRealm {
    pub authority: Pubkey,
    pub governance_token_mint: Pubkey,
    #[max_len(50)]
    pub name: String,
    pub voting_proposal_count: u32,
    pub config: RealmConfig,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct RealmConfig {
    pub min_community_weight_to_create_proposal: u64,
    pub voting_base_time: u32,
    pub community_vote_threshold: VoteThreshold,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum VoteThreshold {
    YesVotePercentage(u8),
    QuorumPercentage(u8),
}

impl GovernanceRealm {
    pub fn can_create_proposal(&self, token_amount: u64) -> bool {
        token_amount >= self.config.min_community_weight_to_create_proposal
    }
}
