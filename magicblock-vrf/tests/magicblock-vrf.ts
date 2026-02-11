import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { GetCommitmentSignature } from "@magicblock-labs/ephemeral-rollups-sdk";
import { MagicblockVrf } from "../target/types/magicblock_vrf";

const DEFAULT_QUEUE = new PublicKey(
  "Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh"
);
const DEFAULT_EPHEMERAL_QUEUE = new PublicKey(
  "5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc"
);

describe("magicblock-vrf", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.EPHEMERAL_PROVIDER_ENDPOINT ||
        "https://devnet.magicblock.app/",
      {
        wsEndpoint:
          process.env.EPHEMERAL_WS_ENDPOINT || "wss://devnet.magicblock.app/",
      }
    ),
    anchor.Wallet.local()
  );

  const program = anchor.workspace.magicblockVrf as Program<MagicblockVrf>;

  const userAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user"), anchor.Wallet.local().publicKey.toBuffer()],
    program.programId
  )[0];

  before(async function () {
    const balance = await provider.connection.getBalance(
      anchor.Wallet.local().publicKey
    );
    console.log("------------------------------------------------------------");
    console.log("  Base Layer RPC   :", provider.connection.rpcEndpoint);
    console.log(
      "  Ephemeral RPC    :",
      providerEphemeralRollup.connection.rpcEndpoint
    );
    console.log(
      "  Wallet           :",
      anchor.Wallet.local().publicKey.toBase58()
    );
    console.log("  Balance          :", balance / LAMPORTS_PER_SOL, "SOL");
    console.log("  User Account PDA :", userAccount.toBase58());
    console.log(
      "------------------------------------------------------------\n"
    );
  });

  it("Initialize user account", async () => {
    try {
      const tx = await program.methods
        .initialize()
        .accountsPartial({
          user: anchor.Wallet.local().publicKey,
          userAccount: userAccount,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      console.log("  tx           :", tx);
    } catch {
      console.log("  Account already exists, skipping.");
    }
  });

  it("[Task 1] Request randomness on base layer", async () => {
    const tx = await program.methods
      .requestRandomness()
      .accountsPartial({ oracleQueue: DEFAULT_QUEUE })
      .rpc({ skipPreflight: true });

    console.log("  tx           :", tx);
    console.log("  Waiting 5s for oracle callback...");
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const account = await program.account.userAccount.fetch(userAccount);
    console.log("  Random value :", account.data.toString());
  });

  it("Delegate account to ephemeral rollup", async () => {
    const tx = await program.methods
      .delegate()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        validator: new PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57"),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("  tx           :", tx);
  });

  it("[Task 2] Request randomness inside ephemeral rollup", async () => {
    const ephemeralProgram = new anchor.Program(
      program.idl,
      providerEphemeralRollup
    ) as typeof program;

    let tx = await ephemeralProgram.methods
      .requestRandomness()
      .accountsPartial({ oracleQueue: DEFAULT_EPHEMERAL_QUEUE })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;
    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], {
      skipPreflight: false,
    });

    console.log("  tx           :", txHash);
    console.log("  Waiting 5s for oracle callback...");
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const accountInfo = await providerEphemeralRollup.connection.getAccountInfo(
      userAccount
    );
    if (accountInfo) {
      const randomValue = new anchor.BN(accountInfo.data.slice(40, 48), "le");
      console.log("  Random value :", randomValue.toString());
    }
  });

  it("Commit and undelegate from ephemeral rollup", async () => {
    let tx = await program.methods
      .undelegate()
      .accounts({ user: providerEphemeralRollup.wallet.publicKey })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;
    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], {
      skipPreflight: false,
    });
    await GetCommitmentSignature(txHash, providerEphemeralRollup.connection);

    console.log("  tx           :", txHash);
  });

  it("Update state on base layer", async () => {
    const tx = await program.methods
      .update(new anchor.BN(45))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
      })
      .rpc();

    const account = await program.account.userAccount.fetch(userAccount);
    console.log("  tx           :", tx);
    console.log("  data         :", account.data.toNumber());
  });

  it("Close user account", async () => {
    const tx = await program.methods
      .close()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("  tx           :", tx);
  });
});
