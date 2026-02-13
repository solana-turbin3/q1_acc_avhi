/**
 * End-to-end devnet test:
 *   1. Create mint_a and mint_b
 *   2. Mint tokens to maker's ATAs
 *   3. Call `make` to open an escrow (expires in ~2 min)
 *   4. Register program PDA as queue authority
 *   5. Call `schedule` to queue the TukTuk auto-refund task
 *   6. Print all addresses so we can watch on explorer
 */

import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { init as initTuktuk, taskQueueAuthorityKey } from "@helium/tuktuk-sdk";
import { TuktukEscrow } from "../target/types/tuktuk_escrow";

const TASK_QUEUE = new PublicKey("UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2");
const TUKTUK_PROGRAM_ID = new PublicKey(
  "tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA"
);

async function main() {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet as anchor.Wallet;
  const payer = wallet.payer;
  const connection = provider.connection;

  console.log("Wallet:", wallet.publicKey.toBase58());

  const program = anchor.workspace.tuktukEscrow as Program<TuktukEscrow>;
  const tuktukProgram = await initTuktuk(provider);

  console.log("\n[1] Creating mints...");
  const mintA = await createMint(connection, payer, payer.publicKey, null, 6);
  const mintB = await createMint(connection, payer, payer.publicKey, null, 6);
  console.log("mint_a:", mintA.toBase58());
  console.log("mint_b:", mintB.toBase58());

  console.log("\n[2] Creating ATAs and minting...");
  const makerAtaA = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mintA,
    payer.publicKey
  );
  await mintTo(
    connection,
    payer,
    mintA,
    makerAtaA.address,
    payer,
    1_000_000_000
  );
  console.log(
    "maker_ata_a:",
    makerAtaA.address.toBase58(),
    "— minted 1000 tokens"
  );

  const seed = new BN(Date.now());
  const seedBuf = seed.toArrayLike(Buffer, "le", 8);
  const [escrowKey] = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), payer.publicKey.toBuffer(), seedBuf],
    program.programId
  );
  const vault = getAssociatedTokenAddressSync(mintA, escrowKey, true);
  console.log("\n[3] Escrow:", escrowKey.toBase58());
  console.log("    Vault :", vault.toBase58());

  console.log("\n[4] Calling make...");
  const makeTx = await program.methods
    .make(seed, new BN(100_000), new BN(50_000))
    .accounts({
      maker: payer.publicKey,
      mintA,
      mintB,
    })
    .rpc({ skipPreflight: true, commitment: "confirmed" });

  console.log("make tx:", makeTx);

  const escrowData = await program.account.escrow.fetch(escrowKey);
  const expiresAt = new Date(escrowData.expiresAt.toNumber() * 1000);
  console.log("expires_at:", expiresAt.toISOString(), "(~20s from now)");

  console.log("\n[5] Checking queue authority registration...");
  const [queueAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("queue_authority")],
    program.programId
  );
  console.log("queue_authority PDA:", queueAuthority.toBase58());

  const tqAuthPda = taskQueueAuthorityKey(TASK_QUEUE, queueAuthority)[0];
  const tqAuthInfo = await connection.getAccountInfo(tqAuthPda);

  if (!tqAuthInfo) {
    console.log("Registering...");
    const regTx = await tuktukProgram.methods
      .addQueueAuthorityV0()
      .accounts({
        payer: payer.publicKey,
        queueAuthority,
        taskQueue: TASK_QUEUE,
      })
      .rpc({ skipPreflight: true, commitment: "confirmed" });
    console.log("Registered — tx:", regTx);
  } else {
    console.log("Already registered.");
  }

  console.log("\n[6] Calling schedule...");

  const tqRaw = (await tuktukProgram.account.taskQueueV0.fetch(
    TASK_QUEUE
  )) as any;
  let taskId = 0;
  for (let i = 0; i < tqRaw.taskBitmap.length; i++) {
    if (tqRaw.taskBitmap[i] !== 0xff) {
      const byte = tqRaw.taskBitmap[i];
      for (let bit = 0; bit < 8; bit++) {
        if ((byte & (1 << bit)) === 0) {
          taskId = i * 8 + bit;
          break;
        }
      }
      break;
    }
  }

  const taskIdBuf = Buffer.alloc(2);
  taskIdBuf.writeUInt16LE(taskId);
  const [taskAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("task"), TASK_QUEUE.toBuffer(), taskIdBuf],
    TUKTUK_PROGRAM_ID
  );

  const [tqAuthorityPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("task_queue_authority"),
      TASK_QUEUE.toBuffer(),
      queueAuthority.toBuffer(),
    ],
    TUKTUK_PROGRAM_ID
  );

  console.log("task_id:", taskId);
  console.log("task account:", taskAccount.toBase58());

  const scheduleTx = await program.methods
    .schedule(taskId)
    .accounts({
      maker: payer.publicKey,
      mintA,
      escrow: escrowKey,
      vault,
      taskQueue: TASK_QUEUE,
      taskQueueAuthority: tqAuthorityPda,
      task: taskAccount,
      tuktukProgram: TUKTUK_PROGRAM_ID,
    } as any)
    .rpc({ skipPreflight: false, commitment: "confirmed" });

  console.log("schedule tx:", scheduleTx);

  console.log("Summary:");
  console.log("Program    :", program.programId.toBase58());
  console.log("Escrow     :", escrowKey.toBase58());
  console.log("Vault      :", vault.toBase58());
  console.log("Task queue :", TASK_QUEUE.toBase58());
  console.log("Task acct  :", taskAccount.toBase58());
  console.log("Expires at :", expiresAt.toISOString());
  console.log(
    `\n>>> https://explorer.solana.com/address/${escrowKey.toBase58()}?cluster=devnet`
  );

  console.log("\n[7] Waiting for TukTuk to fire auto_refund...");
  for (let i = 0; i < 30; i++) {
    await new Promise((r) => setTimeout(r, 5000));
    const escrowInfo = await connection.getAccountInfo(escrowKey);
    const now = new Date().toISOString();
    if (!escrowInfo) {
      console.log(`[${now}] Escrow closed, auto_refund fired by TukTuk!`);
      const makerAtaABalance = await connection.getTokenAccountBalance(
        makerAtaA.address
      );
      console.log(
        "Tokens back in maker_ata_a:",
        makerAtaABalance.value.uiAmount
      );
      break;
    }
    console.log(`[${now}] waiting... escrow still open`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
