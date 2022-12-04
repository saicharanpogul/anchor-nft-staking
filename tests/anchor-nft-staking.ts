import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { AnchorNftStaking } from "../target/types/anchor_nft_staking";
import { setupNft } from "./utils/setupNft";
import { PROGRAM_ID as METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import { assert, expect } from "chai";
import { getAccount } from "@solana/spl-token";

describe("anchor-nft-staking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = anchor.workspace.AnchorNftStaking.provider.wallet;

  const program = anchor.workspace
    .AnchorNftStaking as Program<AnchorNftStaking>;

  let delegateAuthPda: PublicKey;
  let stakeStatePda: PublicKey;
  let nft: any;
  let mintAuth: PublicKey;
  let mint: PublicKey;
  let tokenAddress: PublicKey;

  before(async () => {
    ({ nft, delegateAuthPda, stakeStatePda, mint, mintAuth, tokenAddress } =
      await setupNft(program, wallet.payer));
  });

  it("Stakes", async () => {
    try {
      await program.methods
        .stake()
        .accounts({
          nftTokenAccount: nft.tokenAddress,
          nftMint: nft.mintAddress,
          nftEdition: nft.masterEditionAddress,
          stakeState: stakeStatePda,
          programAuthority: delegateAuthPda,
          user: wallet.publicKey,
          metadataProgram: METADATA_PROGRAM_ID,
        })
        .rpc();
      const account = await program.account.userStakeInfo.fetch(stakeStatePda);
      console.log("After Staking", account);
      expect(account.stakeState === "Staked");
    } catch (error) {
      console.error(error);
      assert(false, error);
    }
  });

  it("Redeems", async () => {
    try {
      await program.methods
        .redeem()
        .accounts({
          nftTokenAccount: nft.tokenAddress,
          stakeMint: mint,
          userStakeAta: tokenAddress,
          user: wallet.publicKey,
          stakeAuthority: mintAuth,
          stakeState: stakeStatePda,
        })
        .rpc();
      const account = await program.account.userStakeInfo.fetch(stakeStatePda);
      console.log("After Redeeming", account);
      expect(account.stakeState === "Staked");
      // const tokenAccount = await getAccount(provider.connection, tokenAddress);
      // console.log(tokenAccount);
    } catch (error) {
      console.error(error);
      assert(false, error);
    }
  });

  it("Unstakes", async () => {
    try {
      await program.methods
        .unstake()
        .accounts({
          nftTokenAccount: nft.tokenAddress,
          nftMint: nft.mintAddress,
          nftEdition: nft.masterEditionAddress,
          metadataProgram: METADATA_PROGRAM_ID,
          stakeMint: mint,
          userStakeAta: tokenAddress,
          user: wallet.publicKey,
          stakeAuthority: mintAuth,
          stakeState: stakeStatePda,
          programAuthority: delegateAuthPda,
        })
        .rpc();
      const account = await program.account.userStakeInfo.fetch(stakeStatePda);
      console.log("After Unstaking", account);
      expect(account.stakeState === "Unstaked");
    } catch (error) {
      console.error(error);
      assert(false, error);
    }
  });
});
