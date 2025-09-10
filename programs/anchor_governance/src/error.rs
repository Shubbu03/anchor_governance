use anchor_lang::prelude::*;

#[error_code]
pub enum GovernanceError {
    #[msg("Insufficient tokens to create proposal")]
    InsufficientTokensToCreateProposal,

    #[msg("Proposal is not in voting state")]
    ProposalNotInVotingState,

    #[msg("Voting period has ended")]
    VotingPeriodEnded,

    #[msg("Voting period has not ended")]
    VotingPeriodNotEnded,

    #[msg("Proposal has not succeeded")]
    ProposalNotSucceeded,

    #[msg("Already voted on this proposal")]
    AlreadyVoted,

    #[msg("Invalid vote threshold")]
    InvalidVoteThreshold,

    #[msg("Proposal execution failed")]
    ProposalExecutionFailed,

    #[msg("Unauthorized action")]
    Unauthorized,

    #[msg("Invalid proposal state transition")]
    InvalidProposalStateTransition,
}
