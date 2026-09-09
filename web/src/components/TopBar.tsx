import { formatAddress } from '../lib/soroban';

function OrbitMark() {
  return (
    <svg width="26" height="26" viewBox="0 0 26 26" fill="none" aria-hidden="true">
      <ellipse cx="13" cy="13" rx="11" ry="5.2" stroke="#7E14FF" strokeWidth="1.4" />
      <circle cx="13" cy="13" r="2.6" fill="#AA3BFF" />
      <circle cx="24" cy="13" r="1.6" fill="#47BFFF" />
    </svg>
  );
}

type Props = {
  walletAddress: string | null;
  onConnect: () => void;
};

export default function TopBar({ walletAddress, onConnect }: Props) {
  return (
    <header className="flex items-center justify-between border-b border-hairline px-6 py-5 sm:px-10">
      <div className="flex items-center gap-3">
        <OrbitMark />
        <div className="leading-tight">
          <p className="text-lg font-semibold tracking-tight">OrbitStream</p>
          <p className="text-xs text-muted">continuous funding, per second</p>
        </div>
      </div>

      <button
        onClick={onConnect}
        className="rounded-full border border-hairline px-4 py-2 text-sm text-ink transition-colors hover:border-violet-bright hover:text-violet-bright"
      >
        {walletAddress ? (
          <span className="flex items-center gap-2 font-mono">
            <span className="h-1.5 w-1.5 rounded-full bg-cyan" />
            {formatAddress(walletAddress)}
          </span>
        ) : (
          'Connect wallet'
        )}
      </button>
    </header>
  );
}
