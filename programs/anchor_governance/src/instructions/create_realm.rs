use crate::constants::*;
use crate::error::GovernanceError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateRealm<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + GovernanceRealm::INIT_SPACE,
        seeds = [GOVERNANCE_REALM_SEED.as_bytes(), name.as_bytes()],
        bump
    )]
    pub governance_realm: Account<'info, GovernanceRealm>,

    pub governance_token_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateRealm<'info> {
    pub fn create_realm(&mut self, name: String, config: RealmConfig) -> Result<()> {
        // Validate config parameters
        require!(
            config.voting_base_time >= MIN_VOTING_TIME
                && config.voting_base_time <= MAX_VOTING_TIME,
            GovernanceError::InvalidVoteThreshold
        );

        // Validate vote threshold values
        match config.community_vote_threshold {
            VoteThreshold::YesVotePercentage(percentage) => {
                require!(
                    percentage >= MIN_YES_VOTE_THRESHOLD && percentage <= MAX_YES_VOTE_THRESHOLD,
                    GovernanceError::InvalidVoteThreshold
                );
            }
            VoteThreshold::QuorumPercentage(percentage) => {
                require!(
                    percentage >= MIN_YES_VOTE_THRESHOLD && percentage <= MAX_YES_VOTE_THRESHOLD,
                    GovernanceError::InvalidVoteThreshold
                );
            }
        }

        self.governance_realm.set_inner(GovernanceRealm {
            authority: self.authority.key(),
            governance_token_mint: self.governance_token_mint.key(),
            name: name.clone(),
            voting_proposal_count: 0,
            config,
            bump: self.governance_realm.bump,
        });

        Ok(())
    }
}
