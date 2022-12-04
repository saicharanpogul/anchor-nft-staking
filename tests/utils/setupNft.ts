import {
  bundlrStorage,
  keypairIdentity,
  Metaplex,
} from "@metaplex-foundation/js";
import { createMint, getAssociatedTokenAddress } from "@solana/spl-token";
import * as anchor from "@project-serum/anchor";
import { Keypair } from "@solana/web3.js";
import { AnchorNftStaking } from "../../target/types/anchor_nft_staking";

export const setupNft = async (
  program: anchor.Program<AnchorNftStaking>,
  payer: Keypair
) => {
  const metaplex = Metaplex.make(program.provider.connection)
    .use(keypairIdentity(payer))
    .use(bundlrStorage());
  const nft = await metaplex.nfts().create({
    name: "Test NFT",
    uri: "",
    sellerFeeBasisPoints: 0,
  });
  console.log("NFT Metadata Pubkey:", nft.metadataAddress.toBase58());
  console.log("NFT Token Address:", nft.tokenAddress.toBase58());
  console.log("NFT Mint Address:", nft.mintAddress.toBase58());
  const [delegateAuthPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  );
  const [stakeStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [payer.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
    program.programId
  );
  console.log("Delegate Authority PDA:", delegateAuthPda.toBase58());
  console.log("Stake State PDA:", stakeStatePda.toBase58());
  const [mintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint")],
    program.programId
  );
  const mint = await createMint(
    program.provider.connection,
    payer,
    mintAuth,
    null,
    2
  );
  console.log("Mint Pubkey:", mint.toBase58());
  const tokenAddress = await getAssociatedTokenAddress(mint, payer.publicKey);
  return {
    nft,
    delegateAuthPda,
    stakeStatePda,
    mint,
    mintAuth,
    tokenAddress,
  };
};
