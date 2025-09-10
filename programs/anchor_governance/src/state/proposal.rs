use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub realm: Pubkey,
    pub proposer: Pubkey,
    #[max_len(100)]
    pub title: String,
    #[max_len(500)]
    pub description: String,
    pub vote_yes: u64,
    pub vote_no: u64,
    pub state: ProposalState,
    pub voting_at: Option<i64>,
    pub voting_completed_at: Option<i64>,
    pub executing_at: Option<i64>,
    #[max_len(10)]
    pub instructions: Vec<ProposalInstruction>,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum ProposalState {
    Draft,
    Voting,
    Succeeded,
    Defeated,
    Executing,
    Completed,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ProposalInstruction {
    pub program_id: Pubkey,
    #[max_len(20)]
    pub accounts: Vec<ProposalAccountMeta>,
    #[max_len(1000)]
    pub data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ProposalAccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl Proposal {
    pub fn can_vote(&self, current_time: i64) -> bool {
        self.state == ProposalState::Voting
            && self
                .voting_at
                .map_or(false, |voting_time| current_time >= voting_time)
    }

    pub fn is_voting_expired(&self, current_time: i64, voting_duration: u32) -> bool {
        self.voting_at.map_or(false, |voting_time| {
            current_time > voting_time + voting_duration as i64
        })
    }

    pub fn calculate_vote_result(
        &self,
        total_supply: u64,
        threshold: &crate::state::governance_realm::VoteThreshold,
    ) -> bool {
        match threshold {
            crate::state::governance_realm::VoteThreshold::YesVotePercentage(percentage) => {
                let total_votes = self.vote_yes + self.vote_no;
                if total_votes == 0 {
                    return false;
                }

                let yes_percentage = (self.vote_yes * 100) / total_votes;
                yes_percentage >= *percentage as u64
            }
            crate::state::governance_realm::VoteThreshold::QuorumPercentage(percentage) => {
                let total_votes = self.vote_yes + self.vote_no;
                let required_quorum = (total_supply * *percentage as u64) / 100;

                total_votes >= required_quorum && self.vote_yes > self.vote_no
            }
        }
    }
}
