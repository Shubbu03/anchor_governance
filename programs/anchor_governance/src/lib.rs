#![allow(unexpected_cfgs, deprecated)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2qtzAcJasoVKCcXQYUkvJikDFUCatGf5bhK9XDNPdWJ1");

#[program]
pub mod anchor_governance {
    use super::*;

    pub fn create_realm(
        ctx: Context<CreateRealm>,
        name: String,
        config: RealmConfig,
    ) -> Result<()> {
        ctx.accounts.create_realm(name, config)
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        instructions: Vec<ProposalInstruction>,
    ) -> Result<()> {
        ctx.accounts.create_proposal(title, description, instructions)
    }
    pub fn start_voting(ctx: Context<StartVoting>) -> Result<()> {
        ctx.accounts.start_voting()
    }

    pub fn cast_vote(ctx: Context<CastVote>, vote: VoteType) -> Result<()> {
        ctx.accounts.cast_vote(vote)
    }

    pub fn finalize_vote(ctx: Context<FinalizeVote>) -> Result<()> {
        ctx.accounts.finalize_vote()
    }

    pub fn execute_proposal<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteProposal<'info>>) -> Result<()> {
        ctx.accounts.execute_proposal(ctx.remaining_accounts)
    }
}
