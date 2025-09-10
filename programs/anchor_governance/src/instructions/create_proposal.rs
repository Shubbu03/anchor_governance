use crate::constants::*;
use crate::error::GovernanceError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
#[instruction(title: String)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(mut)]
    pub governance_realm: Account<'info, GovernanceRealm>,

    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [
            PROPOSAL_SEED.as_bytes(),
            governance_realm.key().as_ref(),
            &governance_realm.voting_proposal_count.to_le_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        constraint = proposer_token_account.mint == governance_realm.governance_token_mint,
        constraint = proposer_token_account.owner == proposer.key()
    )]
    pub proposer_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateProposal<'info> {
    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        instructions: Vec<ProposalInstruction>,
    ) -> Result<()> {
        require!(
            self.governance_realm
                .can_create_proposal(self.proposer_token_account.amount),
            GovernanceError::InsufficientTokensToCreateProposal
        );

        self.proposal.set_inner(Proposal {
            realm: self.governance_realm.key(),
            proposer: self.proposer.key(),
            title,
            description,
            vote_yes: 0,
            vote_no: 0,
            state: ProposalState::Draft,
            voting_at: None,
            voting_completed_at: None,
            executing_at: None,
            instructions,
            bump: self.proposal.bump,
        });

        self.governance_realm.voting_proposal_count += 1;

        Ok(())
    }
}
