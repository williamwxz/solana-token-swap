import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenSwap } from "../target/types/token_swap";
import { 
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
} from "@solana/spl-token";
import { PublicKey } from '@solana/web3.js';

describe("token_swap", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenSwap as Program<TokenSwap>;
  
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;
  let userTokenAAccount: PublicKey;
  let userTokenBAccount: PublicKey;
  let poolTokenAVault: PublicKey;
  let poolTokenBVault: PublicKey;
  let poolAuthority: PublicKey;
  let pool: PublicKey;

  const FEE_RATE = 30; // 0.3%

  it("Initialize", async () => {
    // Create token mints
    tokenAMint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9
    );

    tokenBMint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9
    );

    // Create user token accounts
    userTokenAAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      tokenAMint,
      provider.wallet.publicKey
    );

    userTokenBAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      tokenBMint,
      provider.wallet.publicKey
    );

    // Create pool token vaults
    poolTokenAVault = await createAccount(
      provider.connection,
      provider.wallet.payer,
      tokenAMint,
      provider.wallet.publicKey
    );

    poolTokenBVault = await createAccount(
      provider.connection,
      provider.wallet.payer,
      tokenBMint,
      provider.wallet.publicKey
    );

    // Generate pool and authority
    const [poolKey] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      program.programId
    );
    pool = poolKey;

    const [authorityKey] = PublicKey.findProgramAddressSync(
      [Buffer.from("authority")],
      program.programId
    );
    poolAuthority = authorityKey;

    // Initialize pool
    await program.methods
      .initializePool(new anchor.BN(FEE_RATE))
      .accounts({
        pool: pool,
        tokenAVault: poolTokenAVault,
        tokenBVault: poolTokenBVault,
        authority: poolAuthority,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  });

  it("Deposit", async () => {
    // Mint some tokens to user
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenAMint,
      userTokenAAccount,
      provider.wallet.publicKey,
      1000000000 // 1000 tokens
    );

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenBMint,
      userTokenBAccount,
      provider.wallet.publicKey,
      1000000000 // 1000 tokens
    );

    // Deposit tokens
    await program.methods
      .deposit(
        new anchor.BN(100000000), // 100 tokens A
        new anchor.BN(100000000)  // 100 tokens B
      )
      .accounts({
        pool: pool,
        tokenAVault: poolTokenAVault,
        tokenBVault: poolTokenBVault,
        userTokenA: userTokenAAccount,
        userTokenB: userTokenBAccount,
        user: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  });

  it("Swap", async () => {
    await program.methods
      .swap(
        new anchor.BN(10000000), // 10 tokens in
        new anchor.BN(9000000)   // minimum 9 tokens out
      )
      .accounts({
        pool: pool,
        tokenAVault: poolTokenAVault,
        tokenBVault: poolTokenBVault,
        sourceVault: poolTokenAVault,
        destinationVault: poolTokenBVault,
        poolAuthority: poolAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  });

  it("Withdraw", async () => {
    await program.methods
      .withdraw(
        new anchor.BN(50000000), // 50 tokens A
        new anchor.BN(50000000)  // 50 tokens B
      )
      .accounts({
        pool: pool,
        tokenAVault: poolTokenAVault,
        tokenBVault: poolTokenBVault,
        userTokenA: userTokenAAccount,
        userTokenB: userTokenBAccount,
        poolAuthority: poolAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  });
});