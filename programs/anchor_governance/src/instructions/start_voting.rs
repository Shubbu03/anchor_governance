use crate::error::GovernanceError;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct StartVoting<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    pub governance_realm: Account<'info, GovernanceRealm>,

    #[account(
        mut,
        constraint = proposal.realm == governance_realm.key(),
        constraint = proposal.proposer == proposer.key() @ GovernanceError::Unauthorized,
        constraint = proposal.state == ProposalState::Draft @ GovernanceError::InvalidProposalStateTransition
    )]
    pub proposal: Account<'info, Proposal>,
}

impl<'info> StartVoting<'info> {
    pub fn start_voting(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        let mut proposal_data = self.proposal.clone().into_inner();
        proposal_data.state = ProposalState::Voting;
        proposal_data.voting_at = Some(current_time);

        self.proposal.set_inner(proposal_data);

        Ok(())
    }
}
