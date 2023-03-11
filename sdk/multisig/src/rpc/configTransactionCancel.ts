import {
  Connection,
  PublicKey,
  SendOptions,
  Signer,
  TransactionSignature,
} from "@solana/web3.js";
import * as transactions from "../transactions";
import { translateAndThrowAnchorError } from "../errors";

/** Cancel a config transaction on behalf of the `member`. */
export async function configTransactionCancel({
  connection,
  feePayer,
  member,
  multisigPda,
  transactionIndex,
  memo,
  sendOptions,
}: {
  connection: Connection;
  feePayer: Signer;
  member: Signer;
  multisigPda: PublicKey;
  transactionIndex: bigint;
  memo?: string;
  sendOptions?: SendOptions;
}): Promise<TransactionSignature> {
  const blockhash = (await connection.getLatestBlockhash()).blockhash;

  const tx = transactions.configTransactionCancel({
    blockhash,
    feePayer: feePayer.publicKey,
    multisigPda,
    transactionIndex,
    member: member.publicKey,
    memo,
  });

  tx.sign([feePayer, member]);

  try {
    return await connection.sendTransaction(tx, sendOptions);
  } catch (err) {
    translateAndThrowAnchorError(err);
  }
}