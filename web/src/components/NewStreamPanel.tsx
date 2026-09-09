export default function NewStreamPanel() {
  return (
    <div className="py-6">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold text-ink">Start a stream</h2>
        <span className="rounded-full border border-hairline px-2 py-0.5 text-[11px] text-muted">Coming soon</span>
      </div>
      <p className="mt-1 text-xs text-muted">Fund a maintainer continuously, one second at a time.</p>

      <div className="mt-4 space-y-3 opacity-40">
        <input
          disabled
          type="text"
          placeholder="Receiver address (G...)"
          className="w-full rounded-lg border border-hairline bg-transparent px-3 py-2.5 font-mono text-sm text-ink placeholder:text-muted"
        />
        <input
          disabled
          type="number"
          placeholder="Flow rate (units / sec)"
          className="w-full rounded-lg border border-hairline bg-transparent px-3 py-2.5 font-mono text-sm text-ink placeholder:text-muted"
        />
        <button disabled className="w-full cursor-not-allowed rounded-lg bg-surface2 py-2.5 text-sm font-medium text-muted">
          Start stream
        </button>
      </div>
    </div>
  );
}
