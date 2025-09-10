import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorGovernance } from "../target/types/anchor_governance";
import {
  PublicKey,
  Keypair,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { expect } from "chai";

describe("anchor_governance", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorGovernance as Program<AnchorGovernance>;
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  // Test accounts
  let governanceTokenMint: PublicKey;
  let authority: Keypair;
  let proposer: Keypair;
  let voter1: Keypair;
  let voter2: Keypair;

  // Token accounts
  let authorityTokenAccount: PublicKey;
  let proposerTokenAccount: PublicKey;
  let voter1TokenAccount: PublicKey;
  let voter2TokenAccount: PublicKey;

  // PDAs
  let governanceRealm: PublicKey;
  let proposal: PublicKey;
  let voteRecord1: PublicKey;
  let voteRecord2: PublicKey;

  const realmName = "Test DAO";
  const proposalTitle = "Test Proposal";
  const proposalDescription = "This is a test proposal for the DAO";

  before(async () => {
    // Create test keypairs
    authority = Keypair.generate();
    proposer = Keypair.generate();
    voter1 = Keypair.generate();
    voter2 = Keypair.generate();

    // Airdrop SOL to test accounts
    await connection.requestAirdrop(authority.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await connection.requestAirdrop(proposer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await connection.requestAirdrop(voter1.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await connection.requestAirdrop(voter2.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);

    // Wait for airdrops
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Create governance token mint
    governanceTokenMint = await createMint(
      connection,
      wallet.payer,
      authority.publicKey,
      null,
      6
    );

    // Create token accounts
    authorityTokenAccount = await createAssociatedTokenAccount(
      connection,
      wallet.payer,
      governanceTokenMint,
      authority.publicKey
    );

    proposerTokenAccount = await createAssociatedTokenAccount(
      connection,
      wallet.payer,
      governanceTokenMint,
      proposer.publicKey
    );

    voter1TokenAccount = await createAssociatedTokenAccount(
      connection,
      wallet.payer,
      governanceTokenMint,
      voter1.publicKey
    );

    voter2TokenAccount = await createAssociatedTokenAccount(
      connection,
      wallet.payer,
      governanceTokenMint,
      voter2.publicKey
    );

    // Mint tokens
    await mintTo(connection, wallet.payer, governanceTokenMint, authorityTokenAccount, authority, 1000000);
    await mintTo(connection, wallet.payer, governanceTokenMint, proposerTokenAccount, authority, 100000);
    await mintTo(connection, wallet.payer, governanceTokenMint, voter1TokenAccount, authority, 50000);
    await mintTo(connection, wallet.payer, governanceTokenMint, voter2TokenAccount, authority, 30000);

    // Derive PDAs
    [governanceRealm] = PublicKey.findProgramAddressSync(
      [Buffer.from("governance_realm"), Buffer.from(realmName)],
      program.programId
    );
  });

  describe("Positive Tests", () => {
    it("Creates a governance realm", async () => {
      const config = {
        minCommunityWeightToCreateProposal: new anchor.BN(1000),
        votingBaseTime: 7200, // 2 hours - well above minimum
        communityVoteThreshold: { yesVotePercentage: 60 }
      };

      await program.methods
        .createRealm(realmName, config)
        .accountsPartial({
          authority: authority.publicKey,
          governanceRealm,
          governanceTokenMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([authority])
        .rpc();

      const realmAccount = await program.account.governanceRealm.fetch(governanceRealm);
      expect(realmAccount.name).to.equal(realmName);
      expect(realmAccount.authority.toString()).to.equal(authority.publicKey.toString());
      expect(realmAccount.votingProposalCount).to.equal(0);
    });

    it("Creates a proposal", async () => {
      [proposal] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("proposal"),
          governanceRealm.toBuffer(),
          new anchor.BN(0).toArrayLike(Buffer, "le", 4)
        ],
        program.programId
      );

      const instructions = [];

      await program.methods
        .createProposal(proposalTitle, proposalDescription, instructions)
        .accountsPartial({
          proposer: proposer.publicKey,
          governanceRealm,
          proposal,
          proposerTokenAccount,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([proposer])
        .rpc();

      const proposalAccount = await program.account.proposal.fetch(proposal);
      expect(proposalAccount.title).to.equal(proposalTitle);
      expect(proposalAccount.state).to.deep.equal({ draft: {} });
      expect(proposalAccount.proposer.toString()).to.equal(proposer.publicKey.toString());
    });

    it("Starts voting and casts votes", async () => {
      // Start voting
      await program.methods
        .startVoting()
        .accountsPartial({
          proposer: proposer.publicKey,
          governanceRealm,
          proposal,
        })
        .signers([proposer])
        .rpc();

      // Cast votes
      [voteRecord1] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_record"),
          proposal.toBuffer(),
          voter1.publicKey.toBuffer()
        ],
        program.programId
      );

      await program.methods
        .castVote({ yes: {} })
        .accountsPartial({
          voter: voter1.publicKey,
          governanceRealm,
          proposal,
          voteRecord: voteRecord1,
          voterTokenAccount: voter1TokenAccount,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([voter1])
        .rpc();

      const proposalAccount = await program.account.proposal.fetch(proposal);
      expect(proposalAccount.state).to.deep.equal({ voting: {} });
      expect(proposalAccount.voteYes.toString()).to.equal("50000");
    });
  });

  describe("Negative Tests", () => {
    it("Fails to create proposal with insufficient tokens", async () => {
      const insufficientUser = Keypair.generate();
      await connection.requestAirdrop(insufficientUser.publicKey, anchor.web3.LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 500));

      const insufficientTokenAccount = await createAssociatedTokenAccount(
        connection,
        wallet.payer,
        governanceTokenMint,
        insufficientUser.publicKey
      );

      // Mint only 500 tokens (less than required 1000)
      await mintTo(connection, wallet.payer, governanceTokenMint, insufficientTokenAccount, authority, 500);

      const [failProposal] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("proposal"),
          governanceRealm.toBuffer(),
          new anchor.BN(1).toArrayLike(Buffer, "le", 4)
        ],
        program.programId
      );

      try {
        await program.methods
          .createProposal("Fail Proposal", "Should fail", [])
          .accountsPartial({
            proposer: insufficientUser.publicKey,
            governanceRealm,
            proposal: failProposal,
            proposerTokenAccount: insufficientTokenAccount,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([insufficientUser])
          .rpc();

        expect.fail("Should have failed with insufficient tokens");
      } catch (error) {
        expect(error.message).to.include("InsufficientTokensToCreateProposal");
      }
    });

    it("Fails to start voting by non-proposer", async () => {
      try {
        await program.methods
          .startVoting()
          .accountsPartial({
            proposer: voter1.publicKey, // Wrong proposer
            governanceRealm,
            proposal,
          })
          .signers([voter1])
          .rpc();

        expect.fail("Should have failed with unauthorized");
      } catch (error) {
        expect(error.message).to.include("Unauthorized");
      }
    });
  });

  describe("Complete Flow Test", () => {
    it("Complete governance flow: create realm -> proposal -> vote -> finalize", async () => {
      const flowRealmName = "Flow DAO";

      // 1. Create Realm
      const [flowRealm] = PublicKey.findProgramAddressSync(
        [Buffer.from("governance_realm"), Buffer.from(flowRealmName)],
        program.programId
      );

      const flowConfig = {
        minCommunityWeightToCreateProposal: new anchor.BN(1000),
        votingBaseTime: 3600, // 1 hour minimum
        communityVoteThreshold: { yesVotePercentage: 51 }
      };

      await program.methods
        .createRealm(flowRealmName, flowConfig)
        .accountsPartial({
          authority: authority.publicKey,
          governanceRealm: flowRealm,
          governanceTokenMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([authority])
        .rpc();

      // 2. Create Proposal
      const [flowProposal] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("proposal"),
          flowRealm.toBuffer(),
          new anchor.BN(0).toArrayLike(Buffer, "le", 4)
        ],
        program.programId
      );

      await program.methods
        .createProposal("Flow Proposal", "End-to-end test", [])
        .accountsPartial({
          proposer: proposer.publicKey,
          governanceRealm: flowRealm,
          proposal: flowProposal,
          proposerTokenAccount,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([proposer])
        .rpc();

      // 3. Start Voting
      await program.methods
        .startVoting()
        .accountsPartial({
          proposer: proposer.publicKey,
          governanceRealm: flowRealm,
          proposal: flowProposal,
        })
        .signers([proposer])
        .rpc();

      // 4. Cast Vote
      const [flowVoteRecord] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_record"),
          flowProposal.toBuffer(),
          voter1.publicKey.toBuffer()
        ],
        program.programId
      );

      await program.methods
        .castVote({ yes: {} })
        .accountsPartial({
          voter: voter1.publicKey,
          governanceRealm: flowRealm,
          proposal: flowProposal,
          voteRecord: flowVoteRecord,
          voterTokenAccount: voter1TokenAccount,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([voter1])
        .rpc();

      // Verify the complete flow worked up to voting
      const finalProposal = await program.account.proposal.fetch(flowProposal);
      expect(finalProposal.state).to.deep.equal({ voting: {} });
      expect(finalProposal.voteYes.toString()).to.equal("50000");
      expect(finalProposal.votingAt).to.not.be.null;

      const finalVoteRecord = await program.account.voteRecord.fetch(flowVoteRecord);
      expect(finalVoteRecord.voteType).to.deep.equal({ yes: {} });
      expect(finalVoteRecord.voteWeight.toString()).to.equal("50000");

      // Note: Finalization would require waiting for voting period to end
      // In production, this would be called after the voting time expires
    });
  });
});

