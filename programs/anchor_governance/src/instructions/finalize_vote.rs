use crate::error::GovernanceError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct FinalizeVote<'info> {
    #[account(mut)]
    pub finalizer: Signer<'info>,

    pub governance_realm: Account<'info, GovernanceRealm>,

    #[account(
        mut,
        constraint = proposal.realm == governance_realm.key(),
        constraint = proposal.state == ProposalState::Voting @ GovernanceError::ProposalNotInVotingState
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        constraint = governance_token_mint.key() == governance_realm.governance_token_mint
    )]
    pub governance_token_mint: Account<'info, Mint>,
}

impl<'info> FinalizeVote<'info> {
    pub fn finalize_vote(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            self.proposal
                .is_voting_expired(current_time, self.governance_realm.config.voting_base_time),
            GovernanceError::VotingPeriodNotEnded
        );

        let total_supply = self.governance_token_mint.supply;
        let vote_passed = self.proposal.calculate_vote_result(
            total_supply,
            &self.governance_realm.config.community_vote_threshold,
        );

        let mut proposal_data = self.proposal.clone().into_inner();
        proposal_data.voting_completed_at = Some(current_time);

        if vote_passed {
            proposal_data.state = ProposalState::Succeeded;
        } else {
            proposal_data.state = ProposalState::Defeated;
        }

        self.proposal.set_inner(proposal_data);

        Ok(())
    }
}
