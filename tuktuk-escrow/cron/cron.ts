/**
 * This script creates a recurring TukTuk cron job that fires every minute
 * and attempts to call `auto_refund` on a specific escrow account.
 *
 * TukTuk's crankers will only execute the task if the on-chain
 * `clock.unix_timestamp >= escrow.expires_at` check passes — safe to run
 * the cron more frequently than needed.
 *
 * Usage:
 *   ts-node cron/cron.ts \
 *     --cronName   my-escrow-cron  \
 *     --queueName  my-task-queue   \
 *     --walletPath ~/.config/solana/id.json \
 *     --rpcUrl     https://api.devnet.solana.com \
 *     --maker      <MAKER_PUBKEY> \
 *     --mintA      <MINT_A_PUBKEY> \
 *     --escrowSeed <SEED_AS_U64> \
 *     --fundingAmount 10000000
 */

import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import {
  createCronJob,
  cronJobTransactionKey,
  getCronJobForName,
  init as initCron,
} from "@helium/cron-sdk";
import {
  compileTransaction,
  init,
  taskQueueAuthorityKey,
} from "@helium/tuktuk-sdk";
import {
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { sendInstructions } from "@helium/spl-utils";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { TuktukEscrow } from "../target/types/tuktuk_escrow";

const ESCROW_SEED = Buffer.from("escrow");
const PROGRAM_ID = new PublicKey(
  "92t1k1s6XLTzrFzKvHFRHVX8At6DuzP9BSzkXT33pHjA"
);

function deriveEscrow(maker: PublicKey, seed: bigint): PublicKey {
  const seedBuf = Buffer.alloc(8);
  seedBuf.writeBigUInt64LE(seed);
  return PublicKey.findProgramAddressSync(
    [ESCROW_SEED, maker.toBuffer(), seedBuf],
    PROGRAM_ID
  )[0];
}

async function main() {
  const argv = await yargs(hideBin(process.argv))
    .options({
      cronName: {
        type: "string",
        description: "Name for the cron job",
        demandOption: true,
      },
      queueName: {
        type: "string",
        description: "Name of the TukTuk task queue to use",
        demandOption: true,
      },
      walletPath: {
        type: "string",
        description: "Path to the payer keypair file",
        demandOption: true,
      },
      rpcUrl: {
        type: "string",
        description: "Solana RPC URL (devnet / mainnet)",
        demandOption: true,
      },
      maker: {
        type: "string",
        description: "Public key of the escrow maker",
        demandOption: true,
      },
      mintA: {
        type: "string",
        description: "Public key of mint_a (the deposited token mint)",
        demandOption: true,
      },
      escrowSeed: {
        type: "string",
        description: "Numeric seed used when the escrow was created (u64)",
        demandOption: true,
      },
      fundingAmount: {
        type: "number",
        description: "Lamports to fund the cron job with",
        default: 0.02 * LAMPORTS_PER_SOL,
      },
    })
    .help()
    .alias("help", "h").argv;

  const provider = AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet as anchor.Wallet;

  console.log("Wallet        :", wallet.publicKey.toBase58());
  console.log("RPC           :", argv.rpcUrl);

  const tuktukProgram = await init(provider);
  const cronProgram = await initCron(provider);
  const escrowProgram = anchor.workspace.tuktukEscrow as Program<TuktukEscrow>;

  const makerKey = new PublicKey(argv.maker);
  const mintAKey = new PublicKey(argv.mintA);
  const escrowSeed = BigInt(argv.escrowSeed);

  const escrowKey = deriveEscrow(makerKey, escrowSeed);
  const makerAtaA = getAssociatedTokenAddressSync(mintAKey, makerKey);
  const vault = getAssociatedTokenAddressSync(mintAKey, escrowKey, true);

  console.log("Escrow        :", escrowKey.toBase58());
  console.log("Maker ATA (A) :", makerAtaA.toBase58());
  console.log("Vault         :", vault.toBase58());

  const taskQueue = new PublicKey(
    "CMreFdKxT5oeZhiX8nWTGz9PtXM1AMYTh6dGR2UzdtrA"
  );

  const taskQueueAuthorityPda = taskQueueAuthorityKey(
    taskQueue,
    wallet.publicKey
  )[0];
  const info = await provider.connection.getAccountInfo(taskQueueAuthorityPda);

  if (!info) {
    console.log("Registering wallet as queue authority...");
    await tuktukProgram.methods
      .addQueueAuthorityV0()
      .accounts({
        payer: wallet.publicKey,
        queueAuthority: wallet.publicKey,
        taskQueue,
      })
      .rpc({ skipPreflight: true });
    console.log("Queue authority registered.");
  } else {
    console.log("Queue authority already registered.");
  }

  const autoRefundIx = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: makerKey, isSigner: false, isWritable: true },
      { pubkey: mintAKey, isSigner: false, isWritable: false },
      { pubkey: makerAtaA, isSigner: false, isWritable: true },
      { pubkey: escrowKey, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: escrowProgram.coder.instruction.encode("autoRefund", {}),
  });

  const { transaction, remainingAccounts } = compileTransaction(
    [autoRefundIx],
    []
  );

  let cronJob = await getCronJobForName(cronProgram, argv.cronName);

  if (!cronJob) {
    console.log("Creating cron job:", argv.cronName);

    const {
      pubkeys: { cronJob: cronJobPubkey },
    } = await (
      await createCronJob(cronProgram, {
        tuktukProgram: tuktukProgram,
        taskQueue,
        args: {
          name: argv.cronName,
          schedule: "0 * * * * *",
          freeTasksPerTransaction: 0,
          numTasksPerQueueCall: 1,
        },
      })
    ).rpcAndKeys({ skipPreflight: false });

    cronJob = cronJobPubkey;

    console.log(
      "Funding cron job with",
      argv.fundingAmount / LAMPORTS_PER_SOL,
      "SOL..."
    );
    await sendInstructions(provider, [
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: cronJob,
        lamports: argv.fundingAmount,
      }),
    ]);

    await cronProgram.methods
      .addCronTransactionV0({
        index: 0,
        transactionSource: { compiledV0: [transaction] },
      })
      .accounts({
        payer: provider.publicKey,
        cronJob,
        cronJobTransaction: cronJobTransactionKey(cronJob, 0)[0],
      })
      .remainingAccounts(remainingAccounts)
      .rpc({ skipPreflight: true });

    console.log("Cron job created:", cronJob.toBase58());
  } else {
    console.log("Cron job already exists:", cronJob.toBase58());
  }

  console.log("Cron job  :", cronJob.toBase58());
  console.log("Task queue:", taskQueue.toBase58());
  console.log(
    "\nThe auto_refund instruction will be posted every minute.",
    "\nTukTuk crankers will execute it once the escrow expires."
  );
  console.log(
    "\nTo stop the cron job:\n",
    `  tuktuk -u ${argv.rpcUrl} -w ${argv.walletPath} cron-transaction close --cron-name ${argv.cronName} --id 0\n`,
    `  tuktuk -u ${argv.rpcUrl} -w ${argv.walletPath} cron close --cron-name ${argv.cronName}`
  );
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
