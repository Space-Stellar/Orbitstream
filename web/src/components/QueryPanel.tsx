type Props = {
  senderQuery: string;
  onSenderQueryChange: (value: string) => void;
  onCheck: () => void;
  isLoading: boolean;
  disabled: boolean;
};

export default function QueryPanel({ senderQuery, onSenderQueryChange, onCheck, isLoading, disabled }: Props) {
  return (
    <div className="border-b border-hairline py-6">
      <h2 className="text-sm font-semibold text-ink">Check what's streaming to you</h2>
      <p className="mt-1 text-xs text-muted">Enter a sender's address to see what's accrued so far.</p>

      <div className="mt-4 space-y-3">
        <input
          type="text"
          value={senderQuery}
          onChange={(e) => onSenderQueryChange(e.target.value)}
          placeholder="Sender address (G...)"
          className="w-full rounded-lg border border-hairline bg-transparent px-3 py-2.5 font-mono text-sm text-ink placeholder:text-muted focus:border-violet-bright"
        />
        <button
          onClick={onCheck}
          disabled={disabled || isLoading}
          className="w-full rounded-lg bg-violet py-2.5 text-sm font-medium text-ink transition-colors hover:bg-violet-bright disabled:cursor-not-allowed disabled:bg-surface2 disabled:text-muted"
        >
          {isLoading ? 'Reading the chain…' : 'Check stream'}
        </button>
      </div>
    </div>
  );
}
