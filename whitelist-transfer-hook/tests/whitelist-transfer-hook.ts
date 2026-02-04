import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createTransferCheckedWithTransferHookInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
} from "@solana/spl-token";
import {
  SendTransactionError,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";

describe("whitelist-transfer-hook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace
    .whitelistTransferHook as Program<WhitelistTransferHook>;

  const mint2022 = anchor.web3.Keypair.generate();

  // Sender token account address
  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    wallet.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Recipient token account address
  const recipient = anchor.web3.Keypair.generate();
  const destinationTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    recipient.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // ExtraAccountMetaList address
  // Store extra accounts required by the custom transfer hook instruction
  const [extraAccountMetaListPDA] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("extra-account-metas"), mint2022.publicKey.toBuffer()],
      program.programId
    );

  // Config PDA
  const [configPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  // Per-address whitelist PDA for the wallet
  const [walletWhitelistEntry] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("whitelist-entry"), wallet.publicKey.toBuffer()],
    program.programId
  );

  it("Initialize Config", async () => {
    const tx = await program.methods
      .initializeConfig()
      .accountsPartial({
        admin: wallet.publicKey,
        config: configPDA,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("\nConfig initialized. Admin:", wallet.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Add user to whitelist", async () => {
    const tx = await program.methods
      .addToWhitelist(wallet.publicKey)
      .accountsPartial({
        admin: wallet.publicKey,
        config: configPDA,
        whitelistEntry: walletWhitelistEntry,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("\nUser added to whitelist:", wallet.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Remove user from whitelist", async () => {
    const tx = await program.methods
      .removeFromWhitelist(wallet.publicKey)
      .accountsPartial({
        admin: wallet.publicKey,
        config: configPDA,
        whitelistEntry: walletWhitelistEntry,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("\nUser removed from whitelist:", wallet.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Re-add user to whitelist", async () => {
    const tx = await program.methods
      .addToWhitelist(wallet.publicKey)
      .accountsPartial({
        admin: wallet.publicKey,
        config: configPDA,
        whitelistEntry: walletWhitelistEntry,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("\nUser re-added to whitelist:", wallet.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Create Mint with Transfer Hook Extension", async () => {
    const tx = await program.methods
      .initMint(9)
      .accountsPartial({
        user: wallet.publicKey,
        mint: mint2022.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([mint2022])
      .rpc();

    console.log("\nMint created:", mint2022.publicKey.toBase58());
    console.log("Transaction Signature:", tx);
  });

  it("Create Token Accounts and Mint Tokens", async () => {
    // 100 tokens
    const amount = 100 * 10 ** 9;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        sourceTokenAccount,
        wallet.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        destinationTokenAccount,
        recipient.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint2022.publicKey,
        sourceTokenAccount,
        wallet.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );

    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );

    console.log("\nTransaction Signature: ", txSig);
  });

  // Account to store extra accounts required by the transfer hook instruction
  it("Create ExtraAccountMetaList Account", async () => {
    const tx = await program.methods
      .initializeTransferHook()
      .accountsPartial({
        payer: wallet.publicKey,
        mint: mint2022.publicKey,
        extraAccountMetaList: extraAccountMetaListPDA,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log(
      "\nExtraAccountMetaList Account created:",
      extraAccountMetaListPDA.toBase58()
    );
    console.log("Transaction Signature:", tx);
  });

  it("Transfer Hook with Extra Account Meta", async () => {
    // 1 tokens
    const amount = 1 * 10 ** 9;
    const amountBigInt = BigInt(amount);

    // Automatically resolves extra accounts from the ExtraAccountMetaList
    const transferInstruction =
      await createTransferCheckedWithTransferHookInstruction(
        provider.connection,
        sourceTokenAccount,
        mint2022.publicKey,
        destinationTokenAccount,
        wallet.publicKey,
        amountBigInt,
        9,
        [],
        "confirmed",
        TOKEN_2022_PROGRAM_ID
      );

    const transaction = new Transaction().add(transferInstruction);

    try {
      // Send the transaction
      const txSig = await sendAndConfirmTransaction(
        provider.connection,
        transaction,
        [wallet.payer],
        { skipPreflight: false }
      );
      console.log("\nTransfer Signature:", txSig);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error("\nTransaction failed:", error.logs);
      } else {
        console.error("\nUnexpected error:", error);
      }
    }
  });
});
