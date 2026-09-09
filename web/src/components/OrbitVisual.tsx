type Props = {
  senderLabel: string;
  receiverLabel: string;
  displayValue: string | null;
  caption: string;
  active: boolean;
};

export default function OrbitVisual({ senderLabel, receiverLabel, displayValue, caption, active }: Props) {
  return (
    <div className="relative flex flex-col items-center gap-8 py-10">
      <svg viewBox="0 0 600 260" className="w-full max-w-xl" role="img" aria-label="Token stream orbit">
        <defs>
          <filter id="glow" x="-60%" y="-60%" width="220%" height="220%">
            <feGaussianBlur stdDeviation="5" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        <path
          id="orbit-path"
          d="M 80,130 A 220,65 0 1,1 520,130 A 220,65 0 1,1 80,130"
          fill="none"
          stroke="#1B1129"
          strokeWidth="1.5"
        />
        <path
          d="M 80,130 A 220,65 0 0,1 520,130"
          fill="none"
          stroke="#7E14FF"
          strokeWidth="1.5"
          opacity={active ? 0.6 : 0.2}
        />

        {active && (
          <>
            <circle r="4" fill="#AA3BFF" filter="url(#glow)">
              <animateMotion dur="3.4s" repeatCount="indefinite" rotate="auto">
                <mpath href="#orbit-path" />
              </animateMotion>
            </circle>
            <circle r="3" fill="#47BFFF" filter="url(#glow)">
              <animateMotion dur="3.4s" begin="1.7s" repeatCount="indefinite" rotate="auto">
                <mpath href="#orbit-path" />
              </animateMotion>
            </circle>
          </>
        )}

        <circle cx="80" cy="130" r="6" fill="#0B0710" stroke="#7E14FF" strokeWidth="2" />
        <text x="80" y="100" textAnchor="middle" fill="#F3EDFF" fontSize="13" fontFamily="'Space Grotesk', sans-serif">
          Sender
        </text>
        <text x="80" y="160" textAnchor="middle" fill="#8B84A0" fontSize="11" fontFamily="'IBM Plex Mono', monospace">
          {senderLabel}
        </text>

        <circle cx="520" cy="130" r="6" fill="#0B0710" stroke="#47BFFF" strokeWidth="2" />
        <text x="520" y="100" textAnchor="middle" fill="#F3EDFF" fontSize="13" fontFamily="'Space Grotesk', sans-serif">
          You
        </text>
        <text x="520" y="160" textAnchor="middle" fill="#8B84A0" fontSize="11" fontFamily="'IBM Plex Mono', monospace">
          {receiverLabel}
        </text>
      </svg>

      <div className="text-center">
        <p className="font-mono text-5xl font-medium tracking-tight text-ink sm:text-6xl">
          {displayValue ?? '—'}
        </p>
        <p className="mt-2 text-sm text-muted">{caption}</p>
      </div>
    </div>
  );
}
