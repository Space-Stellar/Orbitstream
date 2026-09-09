# OrbitStream frontend restyle (v2 — build-verified)

Your screenshot wasn't showing "ordinary" design — it was showing **no**
Tailwind CSS at all. I reproduced the exact bug in a sandbox: `tailwindcss@^4`
can no longer be used directly as a PostCSS plugin, and v4 also stopped
reading custom colors from `tailwind.config.js` the way v3 did. Both are
fixed below, and I actually ran `npx vite build` against a copy of your repo
to confirm it compiles and that the brand colors (#0B0710, #7E14FF, #AA3BFF,
#47BFFF) are present in the output CSS — this is verified, not a guess.

## Files to copy into your web/ folder (overwrite):
  web/index.html
  web/postcss.config.js          <- now uses @tailwindcss/postcss
  web/src/index.css              <- custom colors/fonts now live in @theme
  web/src/App.tsx
  web/src/components/TopBar.tsx
  web/src/components/OrbitVisual.tsx
  web/src/components/QueryPanel.tsx
  web/src/components/NewStreamPanel.tsx
  web/src/lib/soroban.ts

## One file to DELETE:
  web/tailwind.config.js
  (no longer needed — the theme now lives in src/index.css's @theme block,
  which is the v4-native way to define it)

## One new dependency to install:
  npm install @tailwindcss/postcss

## Then, from web/:
  npm run dev

If it still looks unstyled after this, it's almost certainly a stale Vite
cache — stop the dev server, delete web/node_modules/.vite, and restart.
