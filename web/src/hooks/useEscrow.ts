import { useState } from 'react';
import {
  rpc,
  TransactionBuilder,
  Networks,
  nativeToScVal,
  Contract,
} from '@stellar/stellar-sdk';
import { signTransaction } from '@stellar/freighter-api';

const RPC_URL = import.meta.env.VITE_STELLAR_RPC_URL || 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = import.meta.env.VITE_STELLAR_NETWORK_PASSPHRASE || Networks.TESTNET;
const FREIGHTER_NETWORK = NETWORK_PASSPHRASE === Networks.PUBLIC ? 'PUBLIC' : 'TESTNET';

export function useEscrow() {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [rpcError, setRpcError] = useState<string | null>(null);

  /**
   * Deposits tokens into the Escrow Vault to start a new stream.
   *
   * @param contractId The Escrow Contract ID
   * @param receiver The receiver's public key
   * @param token The token contract ID
   * @param amount The total amount to lock up in the vault (in atomic units)
   * @param flowRate The amount of tokens to stream per second (in atomic units)
   * @param publicKey The connected user's public key (sender)
   * @returns The transaction hash if successful
   */
  const deposit = async (
    contractId: string,
    receiver: string,
    token: string,
    amount: string,
    flowRate: string,
    publicKey: string
  ): Promise<string | undefined> => {
    setIsSubmitting(true);
    setRpcError(null);

    try {
      const server = new rpc.Server(RPC_URL);
      const account = await server.getAccount(publicKey);
      const contract = new Contract(contractId);

      // Construct arguments matching the Rust contract signature:
      // deposit(sender: Address, receiver: Address, token: Address, amount: i128, flow_rate: i128)
      const args = [
        nativeToScVal(publicKey, { type: 'address' }),
        nativeToScVal(receiver, { type: 'address' }),
        nativeToScVal(token, { type: 'address' }),
        nativeToScVal(BigInt(amount), { type: 'i128' }),
        nativeToScVal(BigInt(flowRate), { type: 'i128' }),
      ];

      // Build the InvokeHostFunctionOp
      const operation = contract.call('deposit', ...args);

      let tx = new TransactionBuilder(account, {
        fee: '100', // Base fee; will be adjusted by simulation
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(operation)
        .setTimeout(30)
        .build();

      // Simulate the transaction to gather resource metrics and actual fees
      const sim = await server.simulateTransaction(tx);
      if (rpc.Api.isSimulationError(sim)) {
        throw new Error(`Simulation failed: ${sim.error}`);
      }

      // Assemble the transaction with the simulated footprint and fee
      tx = rpc.assembleTransaction(tx, NETWORK_PASSPHRASE, sim).built;

      // Sign the transaction with Freighter Wallet
      let signedXdr;
      try {
        signedXdr = await signTransaction(tx.toXDR(), { network: FREIGHTER_NETWORK });
      } catch (err) {
        throw new Error('Wallet signature rejected by the user.');
      }

      const signedTx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
      
      // Submit the fully authorized transaction to the network
      const response = await server.sendTransaction(signedTx);
      
      if (response.status === 'ERROR') {
        throw new Error(`Transaction submission failed.`);
      }

      // Poll until the transaction is confirmed or rejected in a ledger
      let txStatus = await server.getTransaction(response.hash);
      while (txStatus.status === 'NOT_FOUND') {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        txStatus = await server.getTransaction(response.hash);
      }

      if (txStatus.status === 'SUCCESS') {
        return response.hash;
      } else {
        throw new Error(`Transaction failed in ledger validation.`);
      }
    } catch (err: any) {
      console.error('Deposit error:', err);
      setRpcError(err.message || 'An unknown error occurred during deposit.');
    } finally {
      setIsSubmitting(false);
    }
  };

  /**
   * Claims accrued tokens from a specific stream.
   *
   * @param contractId The Escrow Contract ID
   * @param streamId The ID of the stream
   * @param publicKey The connected user's public key (receiver typically, or whoever invokes)
   * @returns The transaction hash if successful
   */
  const claim = async (
    contractId: string,
    streamId: string,
    publicKey: string
  ): Promise<string | undefined> => {
    setIsSubmitting(true);
    setRpcError(null);

    try {
      const server = new rpc.Server(RPC_URL);
      const account = await server.getAccount(publicKey);
      const contract = new Contract(contractId);

      // Construct arguments matching the Rust contract signature:
      // claim(stream_id: u64)
      const args = [
        nativeToScVal(BigInt(streamId), { type: 'u64' }),
      ];

      const operation = contract.call('claim', ...args);

      let tx = new TransactionBuilder(account, {
        fee: '100',
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(operation)
        .setTimeout(30)
        .build();

      const sim = await server.simulateTransaction(tx);
      if (rpc.Api.isSimulationError(sim)) {
        throw new Error(`Simulation failed: ${sim.error}`);
      }

      tx = rpc.assembleTransaction(tx, NETWORK_PASSPHRASE, sim).built;

      let signedXdr;
      try {
        signedXdr = await signTransaction(tx.toXDR(), { network: FREIGHTER_NETWORK });
      } catch (err) {
        throw new Error('Wallet signature rejected by the user.');
      }

      const signedTx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
      const response = await server.sendTransaction(signedTx);

      if (response.status === 'ERROR') {
        throw new Error(`Transaction submission failed.`);
      }

      let txStatus = await server.getTransaction(response.hash);
      while (txStatus.status === 'NOT_FOUND') {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        txStatus = await server.getTransaction(response.hash);
      }

      if (txStatus.status === 'SUCCESS') {
        return response.hash;
      } else {
        throw new Error(`Transaction failed in ledger validation.`);
      }
    } catch (err: any) {
      console.error('Claim error:', err);
      setRpcError(err.message || 'An unknown error occurred during claim.');
    } finally {
      setIsSubmitting(false);
    }
  };

  return {
    deposit,
    claim,
    isSubmitting,
    rpcError,
  };
}
