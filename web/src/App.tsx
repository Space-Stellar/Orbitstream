import { useEffect, useState } from 'react';
import { Wallet, CheckCircle2 } from 'lucide-react';
import TopBar from './components/TopBar';
import OrbitVisual from './components/OrbitVisual';
import QueryPanel from './components/QueryPanel';
import NewStreamPanel from './components/NewStreamPanel';
import { checkFreighterConnection, connectFreighter, getBalance, getFlowRate, claimTokens, formatAddress } from './lib/soroban';

type StreamStatus =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'not-found' }
  | { kind: 'error' }
  | { kind: 'ready'; balance: bigint; flowRate: bigint | null; syncedAt: number }
  | { kind: 'claimed' };

export default function App() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [senderQuery, setSenderQuery] = useState('');
  const [status, setStatus] = useState<StreamStatus>({ kind: 'idle' });
  const [isClaiming, setIsClaiming] = useState(false);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    checkFreighterConnection().then((addr) => addr && setWalletAddress(addr));
  }, []);

  // Ticks the display forward locally between on-chain checks, using the real
  // flow_rate from get_stream. Purely visual — every explicit "Check stream"
  // re-syncs from the authoritative get_balance read.
  useEffect(() => {
    if (status.kind !== 'ready' || status.flowRate === null) return;
    const id = setInterval(() => setNow(Date.now()), 100);
    return () => clearInterval(id);
  }, [status]);

  const connectWallet = async () => {
    try {
      const addr = await connectFreighter();
      if (addr) setWalletAddress(addr);
    } catch (error) {
      console.error('Connection failed:', error);
    }
  };

  const checkStream = async () => {
    if (!walletAddress || !senderQuery) return;
    setStatus({ kind: 'loading' });
    setTxHash(null);

    try {
      const balance = await getBalance(senderQuery, walletAddress, walletAddress);
      if (balance === null) {
        setStatus({ kind: 'not-found' });
        return;
      }
      const flowRate = await getFlowRate(senderQuery, walletAddress, walletAddress);
      setNow(Date.now());
      setStatus({ kind: 'ready', balance, flowRate, syncedAt: Date.now() });
    } catch (error) {
      console.error('RPC query failed:', error);
      setStatus({ kind: 'error' });
    }
  };

  const claim = async () => {
    if (!walletAddress || !senderQuery) return;
    setIsClaiming(true);
    try {
      const res = await claimTokens(senderQuery, walletAddress);
      if (res.status !== 'ERROR') {
        setTxHash(res.hash);
        setStatus({ kind: 'claimed' });
      } else {
        alert('Transaction failed on-chain.');
      }
    } catch (error) {
      console.error('Claim transaction failed:', error);
      alert('Claim canceled or failed. Check console for details.');
    }
    setIsClaiming(false);
  };

  const liveDisplay = (): string | null => {
    if (status.kind === 'claimed') return '0';
    if (status.kind !== 'ready') return null;
    if (status.flowRate === null) return status.balance.toLocaleString();

    const base = Number(status.balance);
    const flow = Number(status.flowRate);
    if (!Number.isSafeInteger(base) || !Number.isFinite(flow)) return status.balance.toLocaleString();

    const elapsedSeconds = (now - status.syncedAt) / 1000;
    return Math.floor(base + flow * elapsedSeconds).toLocaleString();
  };

  const caption = (): string => {
    switch (status.kind) {
      case 'idle':
        return 'Enter a sender address to see what has accrued';
      case 'loading':
        return 'Reading the chain…';
      case 'not-found':
        return 'No stream found between these two addresses';
      case 'error':
        return 'Could not reach the network — try again';
      case 'claimed':
        return 'Claimed — the stream keeps accruing from here';
      case 'ready':
        return status.flowRate !== null
          ? `Accruing at ${status.flowRate.toLocaleString()} units / sec`
          : 'Accrued so far';
    }
  };

  const canClaim = status.kind === 'ready' && status.balance > 0n;

  return (
    <div className="min-h-screen bg-bg text-ink">
      <TopBar walletAddress={walletAddress} onConnect={connectWallet} />

      <main className="mx-auto grid max-w-6xl grid-cols-1 gap-10 px-6 py-10 sm:px-10 lg:grid-cols-[1fr_320px]">
        <section className="rounded-2xl border border-hairline bg-surface/60 px-6">
          <OrbitVisual
            senderLabel={senderQuery ? formatAddress(senderQuery) : 'not set'}
            receiverLabel={walletAddress ? formatAddress(walletAddress) : 'not connected'}
            displayValue={liveDisplay()}
            caption={caption()}
            active={status.kind === 'ready'}
          />

          {canClaim && (
            <div className="flex flex-col items-center gap-3 border-t border-hairline py-6">
              <button
                onClick={claim}
                disabled={isClaiming}
                className="flex items-center gap-2 rounded-full bg-violet px-6 py-3 text-sm font-medium text-ink transition-colors hover:bg-violet-bright disabled:cursor-not-allowed disabled:bg-surface2 disabled:text-muted"
              >
                <Wallet size={16} />
                {isClaiming ? 'Waiting for signature…' : 'Claim tokens'}
              </button>

              {txHash && (
                <a
                  href={`https://stellar.expert/explorer/testnet/tx/${txHash}`}
                  target="_blank"
                  rel="noreferrer"
                  className="flex items-center gap-2 text-sm text-cyan hover:text-violet-bright"
                >
                  <CheckCircle2 size={14} />
                  View transaction
                </a>
              )}
            </div>
          )}
        </section>

        <aside>
          <QueryPanel
            senderQuery={senderQuery}
            onSenderQueryChange={setSenderQuery}
            onCheck={checkStream}
            isLoading={status.kind === 'loading'}
            disabled={!walletAddress}
          />
          <NewStreamPanel />
        </aside>
      </main>
    </div>
  );
}
