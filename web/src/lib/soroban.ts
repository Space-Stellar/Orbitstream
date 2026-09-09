import { isAllowed, setAllowed, getAddress, signTransaction } from '@stellar/freighter-api';
import {
  rpc,
  Address,
  TransactionBuilder,
  Networks,
  Operation,
  scValToNative,
  xdr,
} from '@stellar/stellar-sdk';

const RPC_URL = 'https://soroban-testnet.stellar.org:443';
const contractId = import.meta.env.VITE_STREAM_CONTRACT_ID;

function server() {
  return new rpc.Server(RPC_URL);
}

export async function checkFreighterConnection(): Promise<string | null> {
  try {
    const allowed = await isAllowed();
    if (allowed.isAllowed) {
      const userInfo = await getAddress();
      return userInfo.address || null;
    }
  } catch {
    console.error('Freighter not detected');
  }
  return null;
}

export async function connectFreighter(): Promise<string | null> {
  let allowed = await isAllowed();
  if (!allowed.isAllowed) {
    await setAllowed();
    allowed = await isAllowed();
  }
  if (allowed.isAllowed) {
    const userInfo = await getAddress();
    return userInfo.address || null;
  }
  return null;
}

async function simulateRead(fn: string, args: xdr.ScVal[], caller: string) {
  const s = server();
  const account = await s.getAccount(caller);

  const tx = new TransactionBuilder(account, { fee: '1000', networkPassphrase: Networks.TESTNET })
    .addOperation(
      Operation.invokeContractFunction({
        contract: contractId,
        function: fn,
        args,
      }),
    )
    .setTimeout(30)
    .build();

  return s.simulateTransaction(tx);
}

/** Authoritative balance read — unchanged from the original working implementation. */
export async function getBalance(sender: string, receiver: string, caller: string) {
  const simulation = await simulateRead(
    'get_balance',
    [new Address(sender).toScVal(), new Address(receiver).toScVal()],
    caller,
  );

  if (rpc.Api.isSimulationSuccess(simulation)) {
    const resultVal = (simulation as rpc.Api.SimulateTransactionSuccessResponse).result!.retval;
    return scValToNative(resultVal) as bigint;
  }
  return null;
}

/**
 * Best-effort read of the stream's flow rate, used only to animate the balance
 * ticking upward locally between on-chain checks. If this fails for any reason,
 * callers should fall back to the static balance from getBalance — the live
 * tick is a display enhancement, never the source of truth.
 */
export async function getFlowRate(sender: string, receiver: string, caller: string): Promise<bigint | null> {
  try {
    const simulation = await simulateRead(
      'get_stream',
      [new Address(sender).toScVal(), new Address(receiver).toScVal()],
      caller,
    );
    if (!rpc.Api.isSimulationSuccess(simulation)) return null;

    const resultVal = (simulation as rpc.Api.SimulateTransactionSuccessResponse).result!.retval;
    const decoded = scValToNative(resultVal);
    const flowRate = decoded instanceof Map ? decoded.get('flow_rate') : decoded?.flow_rate;
    return typeof flowRate === 'bigint' ? flowRate : null;
  } catch (e) {
    console.error('get_stream read failed (non-fatal, ticker will stay static):', e);
    return null;
  }
}

/** Claim flow — unchanged from the original working implementation. */
export async function claimTokens(sender: string, receiver: string) {
  const s = server();
  const account = await s.getAccount(receiver);

  const tx = new TransactionBuilder(account, { fee: '1000', networkPassphrase: Networks.TESTNET })
    .addOperation(
      Operation.invokeContractFunction({
        contract: contractId,
        function: 'claim',
        args: [new Address(sender).toScVal(), new Address(receiver).toScVal()],
      }),
    )
    .setTimeout(30)
    .build();

  const preparedTx = await s.prepareTransaction(tx);
  const { signedTxXdr } = await signTransaction(preparedTx.toXDR(), { networkPassphrase: Networks.TESTNET });
  const signedTx = TransactionBuilder.fromXDR(signedTxXdr, Networks.TESTNET);
  return s.sendTransaction(signedTx);
}

export function formatAddress(address: string) {
  if (!address) return '';
  return `${address.slice(0, 5)}...${address.slice(-4)}`;
}
