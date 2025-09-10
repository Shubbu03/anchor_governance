use crate::constants::*;
use crate::error::GovernanceError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    pub governance_realm: Account<'info, GovernanceRealm>,

    #[account(
        mut,
        constraint = proposal.realm == governance_realm.key(),
        constraint = proposal.state == ProposalState::Voting @ GovernanceError::ProposalNotInVotingState
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + VoteRecord::INIT_SPACE,
        seeds = [
            VOTE_RECORD_SEED.as_bytes(),
            proposal.key().as_ref(),
            voter.key().as_ref()
        ],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,

    #[account(
        constraint = voter_token_account.mint == governance_realm.governance_token_mint,
        constraint = voter_token_account.owner == voter.key()
    )]
    pub voter_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CastVote<'info> {
    pub fn cast_vote(&mut self, vote: VoteType) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            self.proposal.can_vote(current_time),
            GovernanceError::ProposalNotInVotingState
        );

        require!(
            !self
                .proposal
                .is_voting_expired(current_time, self.governance_realm.config.voting_base_time),
            GovernanceError::VotingPeriodEnded
        );

        let vote_weight = self.voter_token_account.amount;

        let mut proposal_data = self.proposal.clone().into_inner();
        match vote {
            VoteType::Yes => proposal_data.vote_yes += vote_weight,
            VoteType::No => proposal_data.vote_no += vote_weight,
            VoteType::Abstain => {} // Abstain votes don't affect totals
        }
        self.proposal.set_inner(proposal_data);

        self.vote_record.set_inner(VoteRecord {
            proposal: self.proposal.key(),
            voter: self.voter.key(),
            vote_weight,
            vote_type: vote,
            voted_at: current_time,
            bump: self.vote_record.bump,
        });

        Ok(())
    }
}
