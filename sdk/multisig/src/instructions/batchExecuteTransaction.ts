import { Connection, PublicKey } from "@solana/web3.js";
import {
  Batch,
  createBatchExecuteTransactionInstruction,
  VaultBatchTransaction,
} from "../generated";
import {
  getBatchTransactionPda,
  getProposalPda,
  getTransactionPda,
  getVaultPda,
} from "../pda";
import { remainingAccountsForTransactionExecute } from "../utils";

export async function batchExecuteTransaction({
  connection,
  multisigPda,
  member,
  batchIndex,
  transactionIndex,
}: {
  connection: Connection;
  multisigPda: PublicKey;
  member: PublicKey;
  batchIndex: bigint;
  transactionIndex: number;
}) {
  const [proposalPda] = getProposalPda({
    multisigPda,
    transactionIndex: batchIndex,
  });
  const [batchPda] = getTransactionPda({
    multisigPda,
    index: batchIndex,
  });
  const [batchTransactionPda] = getBatchTransactionPda({
    multisigPda,
    batchIndex,
    transactionIndex,
  });

  const batchAccount = await Batch.fromAccountAddress(connection, batchPda);
  const [vaultPda] = getVaultPda({
    multisigPda,
    index: batchAccount.vaultIndex,
  });

  const batchTransactionAccount =
    await VaultBatchTransaction.fromAccountAddress(
      connection,
      batchTransactionPda
    );

  const remainingAccounts = await remainingAccountsForTransactionExecute({
    connection,
    message: batchTransactionAccount.message,
    ephemeralSignerBumps: [...batchTransactionAccount.ephemeralSignerBumps],
    vaultPda,
    transactionPda: batchPda,
  });

  return createBatchExecuteTransactionInstruction({
    multisig: multisigPda,
    member,
    proposal: proposalPda,
    batch: batchPda,
    transaction: batchTransactionPda,
    anchorRemainingAccounts: remainingAccounts,
  });
}