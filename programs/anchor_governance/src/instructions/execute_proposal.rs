use crate::error::GovernanceError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,

    pub governance_realm: Account<'info, GovernanceRealm>,

    #[account(
        mut,
        constraint = proposal.realm == governance_realm.key(),
        constraint = proposal.state == ProposalState::Succeeded @ GovernanceError::ProposalNotSucceeded
    )]
    pub proposal: Account<'info, Proposal>,
}

impl<'info> ExecuteProposal<'info> {
    pub fn execute_proposal(&mut self, remaining_accounts: &[AccountInfo<'info>]) -> Result<()> {
        let mut proposal_data = self.proposal.clone().into_inner();
        proposal_data.state = ProposalState::Executing;
        proposal_data.executing_at = Some(Clock::get()?.unix_timestamp);
        self.proposal.set_inner(proposal_data.clone());

        // Execute each instruction in the proposal
        for proposal_instruction in proposal_data.instructions.iter() {
            let mut accounts = Vec::new();

            // Map proposal accounts to actual account infos
            for account_meta in &proposal_instruction.accounts {
                let _account_info = remaining_accounts
                    .iter()
                    .find(|acc| acc.key() == account_meta.pubkey)
                    .ok_or(GovernanceError::ProposalExecutionFailed)?;

                accounts.push(anchor_lang::solana_program::instruction::AccountMeta {
                    pubkey: account_meta.pubkey,
                    is_signer: account_meta.is_signer,
                    is_writable: account_meta.is_writable,
                });
            }

            let instruction = Instruction {
                program_id: proposal_instruction.program_id,
                accounts,
                data: proposal_instruction.data.clone(),
            };

            // Execute the instruction
            invoke(&instruction, remaining_accounts)
                .map_err(|_| GovernanceError::ProposalExecutionFailed)?;
        }

        // Mark proposal as completed
        let mut final_proposal_data = self.proposal.clone().into_inner();
        final_proposal_data.state = ProposalState::Completed;
        self.proposal.set_inner(final_proposal_data);

        Ok(())
    }
}
