#!/usr/bin/env bash
set -euo pipefail

REPO="Space-Stellar/Orbitstream"
DRY_RUN=false

if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
  echo "=== RUNNING IN DRY-RUN MODE (No changes will be applied to GitHub) ==="
fi

# Define labels: name, color, description
declare -A LABELS=(
  ["difficulty: easy"]="0E8A16:Beginner friendly task with clear scope"
  ["difficulty: medium"]="FBCA04:Intermediate task requiring domain familiarity"
  ["difficulty: high"]="B60205:Complex task with architectural or protocol implications"
  ["good first issue"]="7057ff:Good for newcomers"
  ["enhancement"]="a2eeef:New feature or request"
  ["bug"]="d73a4a:Something isn't working"
  ["docs"]="0075ca:Improvements or additions to documentation"
  ["drips-wave"]="1d76db:Tasks tracked for the Drips Stellar Wave program"
)

echo "--- Ensuring Labels on ${REPO} ---"
for label in "${!LABELS[@]}"; do
  IFS=":" read -r color desc <<< "${LABELS[$label]}"
  if [ "$DRY_RUN" = true ]; then
    echo "[DRY-RUN] Would create label: '$label' (color: #$color, description: '$desc')"
  else
    echo "Creating or updating label '$label'..."
    gh label create "$label" --repo "$REPO" --color "$color" --description "$desc" --force 2>/dev/null || true
  fi
done

create_issue() {
  local title="$1"
  local labels="$2"
  local body="$3"

  if [ "$DRY_RUN" = true ]; then
    echo ""
    echo "================================================================================"
    echo "[DRY-RUN ISSUE] Title: $title"
    echo "Labels: $labels"
    echo "--------------------------------------------------------------------------------"
    echo "$body"
    echo "================================================================================"
  else
    echo "Creating issue: $title..."
    gh issue create \
      --repo "$REPO" \
      --title "$title" \
      --body "$body" \
      --label "$labels"
  fi
}

echo ""
echo "--- Creating Issue Backlog (12 Issues) ---"

# Issue 1: Contracts (Stream)
create_issue \
  "fix(stream): prevent trapped stream in cancel_stream when allowance is insufficient" \
  "bug,difficulty: high,drips-wave" \
  "## Summary
In \`contracts/stream/src/lib.rs\`, \`cancel_stream\` calculates \`pending_payout\` and invokes \`token_client.transfer(&sender, &receiver, &(pending_payout as i128))\` to settle accrued balance before closing. If the sender's token balance or allowance drops below \`pending_payout\`, the transfer reverts, rolling back the persistent storage deletion and trapping the stream open.

## Why It Matters
When a sender cannot cover accrued balance, they cannot terminate the stream. The stream continues to accrue virtual debt indefinitely, and the receiver sees a growing phantom balance that can never be claimed.

## Key Files & Modules
* \`contracts/stream/src/lib.rs\` - \`cancel_stream\` settlement logic
* \`contracts/stream/src/test.rs\` - Test suite covering cancellation edge cases

## Edge Cases & Constraints
* The sender must not be able to evade legitimately accrued debt if funds exist.
* If settled amount is clamped to \`min(accrued, available_balance, available_allowance)\`, the shortfall must be accounted for or recorded.
* Storage deletion must happen safely in accordance with Checks-Effects-Interactions.

## Acceptance Criteria
- [ ] \`cancel_stream\` allows terminating an active stream even if sender allowance or balance is below accrued payout.
- [ ] Settles the maximum available transferable balance up to accrued debt without reverting.
- [ ] Persistent storage key \`StreamKey\` is deleted upon cancellation.
- [ ] Cancellation event publishes the actual amount settled.
- [ ] Comprehensive tests in \`contracts/stream/src/test.rs\` assert cancellation behavior with zero balance and zero allowance.

## Validation Steps
1. Run \`cargo test -p stream\` to verify unit tests pass.
2. Simulate a stream where sender balance is 0 and verify \`cancel_stream\` succeeds.

## Tech Stack
Rust, Soroban SDK"

# Issue 2: Contracts (Escrow)
create_issue \
  "feat(escrow): implement deposit top-up mechanism for active streams" \
  "enhancement,difficulty: medium,drips-wave" \
  "## Summary
In \`contracts/escrow/src/lib.rs\`, senders initialize streams via \`deposit\`, which always creates a new numeric \`stream_id\`. There is currently no method allowing a sender to add supplemental deposit capital to an existing active \`stream_id\`.

## Why It Matters
If a funding stream reaches the end of its deposited runway, the sender must wait for it to be cancelled and recreate a new stream with a new \`stream_id\`. Adding a \`top_up\` method allows long-running contributor relationships to continue uninterrupted.

## Key Files & Modules
* \`contracts/escrow/src/lib.rs\` - Escrow vault stream management
* \`contracts/escrow/src/test.rs\` - Escrow test cases

## Edge Cases & Constraints
* \`top_up\` must require authorization from \`stream.sender\` (\`sender.require_auth()\`).
* Only active, un-cancelled streams can be topped up.
* Must use checked addition (\`checked_add\`) when updating \`stream.total_deposited\` to avoid integer overflow.
* Must pull tokens directly into \`env.current_contract_address()\`.

## Acceptance Criteria
- [ ] Public function \`top_up(env: Env, stream_id: u64, amount: i128) -> Result<(), Error>\` implemented in \`EscrowVaultContract\`.
- [ ] Transfers \`amount\` tokens from sender to escrow contract.
- [ ] Increments \`stream.total_deposited\` safely.
- [ ] Reverts with \`Error::StreamNotFound\` if stream does not exist.
- [ ] Reverts with \`Error::InvalidAmount\` if \`amount <= 0\`.
- [ ] Unit tests added in \`contracts/escrow/src/test.rs\` confirming deposit extension.

## Validation Steps
1. \`cargo test -p orbitstream-escrow\` passes.
2. Verify \`top_up\` increases contract token balance and updates stream deposit.

## Tech Stack
Rust, Soroban SDK"

# Issue 3: Contracts (Escrow)
create_issue \
  "feat(escrow): emit soroban events for vault lifecycle transitions" \
  "enhancement,difficulty: medium,drips-wave" \
  "## Summary
Unlike \`contracts/stream/src/lib.rs\` which emits \`init\`, \`claim\`, and \`cancel\` events, \`contracts/escrow/src/lib.rs\` currently emits zero events on \`deposit\`, \`withdraw\`, \`claim\`, or \`cancel_stream\`.

## Why It Matters
Off-chain indexers and user interfaces cannot detect escrow activity without polling full contract storage. Emitting standardized Soroban events allows downstream indexers to track vault lifecycle events in real-time.

## Key Files & Modules
* \`contracts/escrow/src/lib.rs\` - Escrow contract implementation
* \`contracts/escrow/src/test.rs\` - Escrow contract test suite

## Edge Cases & Constraints
* Event topics should follow Soroban conventions: \`(symbol_short!(\"...\"), sender, receiver)\`.
* Event data payloads must supply atomic amount figures and stream IDs.

## Acceptance Criteria
- [ ] \`deposit\` emits \`(symbol_short!(\"deposit\"), sender, receiver)\` with data \`(stream_id, amount, flow_rate)\`.
- [ ] \`withdraw\` emits \`(symbol_short!(\"withdraw\"), sender)\` with data \`(stream_id, amount)\`.
- [ ] \`claim\` emits \`(symbol_short!(\"claim\"), receiver)\` with data \`(stream_id, claimable)\`.
- [ ] \`cancel_stream\` emits \`(symbol_short!(\"cancel\"), caller)\` with data \`(stream_id, claimable, remainder)\`.
- [ ] Unit tests verify event emissions using Soroban test utilities.

## Validation Steps
1. Run \`cargo test -p orbitstream-escrow\`.
2. Inspect \`env.events().all()\` in tests to confirm topics and payloads.

## Tech Stack
Rust, Soroban SDK"

# Issue 4: Indexer
create_issue \
  "feat(indexer): parse claim and cancel events from stream contract" \
  "enhancement,difficulty: medium,drips-wave" \
  "## Summary
\`indexer/app/stellar/parser.py\` only matches \`if event_type_scval.sym == b'init':\`. It completely discards \`claim\` and \`cancel\` events emitted by the stream contract on-chain.

## Why It Matters
The indexer database does not reflect stream terminations or claim settlements, causing local state to diverge from the ledger.

## Key Files & Modules
* \`indexer/app/stellar/parser.py\` - Event payload extraction
* \`indexer/app/db/models.py\` - Database models and schema
* \`indexer/app/stellar/rpc_poller.py\` - RPC event dispatcher

## Edge Cases & Constraints
* Events might arrive out-of-order across re-polling intervals if ledgers are split.
* Must handle zero-value claims and cancellations without database transaction rollbacks.

## Acceptance Criteria
- [ ] Parser decodes \`claim\` symbol and records claim timestamp, receiver address, and claimed stroops.
- [ ] Parser decodes \`cancel\` symbol and flags stream as closed/terminated in SQLite.
- [ ] Database schema in \`models.py\` updated to track stream status and cumulative claimed amounts.
- [ ] Poller logs event discovery for \`claim\` and \`cancel\` types.

## Validation Steps
1. Run \`python indexer/main.py\`.
2. Mock \`claim\` and \`cancel\` XDR events and verify rows update in \`orbitstream.db\`.

## Tech Stack
Python 3.10+, SQLAlchemy, aiohttp, stellar-sdk"

# Issue 5: Indexer
create_issue \
  "test(indexer): add automated test suite for XDR event parsing" \
  "enhancement,difficulty: easy,good first issue,drips-wave" \
  "## Summary
The \`indexer/\` module currently contains no automated tests or test runners. Changes to XDR decoding or database models must be tested manually by running the poller.

## Why It Matters
Without automated tests, regressions in XDR deserialization or database async commits cannot be caught in CI.

## Key Files & Modules
* \`indexer/app/stellar/parser.py\` - Event parsing logic
* \`indexer/requirements.txt\` - Python dependencies
* \`indexer/main.py\` - Indexer entry point

## Edge Cases & Constraints
* Tests must run with an in-memory SQLite database (\`sqlite+aiosqlite:///:memory:\`) without file persistence side-effects.
* Tests must not attempt network connections to live Soroban RPC.

## Acceptance Criteria
- [ ] Add \`pytest\` and \`pytest-asyncio\` to \`indexer/requirements.txt\`.
- [ ] Create \`indexer/tests/test_parser.py\` with test fixtures for Soroban event XDR strings.
- [ ] Verify test suite runs with \`pytest\` and passes.
- [ ] Document test execution in \`CONTRIBUTING.md\`.

## Validation Steps
1. \`cd indexer && pytest\` passes with 100% success.

## Tech Stack
Python 3, Pytest, aiosqlite"

# Issue 6: Indexer
create_issue \
  "feat(indexer): decode real addresses and amounts from soroban event xdr" \
  "bug,difficulty: medium,drips-wave" \
  "## Summary
In \`indexer/app/stellar/parser.py\` (lines 25-29), the parser logs an \`init\` event but populates the database using hardcoded placeholder strings:
\`StreamEvent(sender=\"DECODED_SENDER_ADDRESS\", receiver=\"DECODED_RECEIVER_ADDRESS\", flow_rate=5000)\`.

## Why It Matters
The indexer currently stores dummy strings instead of real Stellar public keys and flow rates from on-chain transactions.

## Key Files & Modules
* \`indexer/app/stellar/parser.py\` - Event parsing function \`process_event\`
* \`indexer/app/db/models.py\` - SQLite \`StreamEvent\` model

## Edge Cases & Constraints
* Topic 1 (\`sender\`) and Topic 2 (\`receiver\`) are SCVal addresses that must be encoded to StrKey format (starting with 'G' or 'C').
* Event data is an SCVal \`u64\` integer for \`flow_rate\`.

## Acceptance Criteria
- [ ] \`process_event\` inspects \`topics[1]\` and \`topics[2]\` to extract authenticated StrKey addresses.
- [ ] \`process_event\` decodes \`event[\"value\"]\` to retrieve the real \`flow_rate\` integer.
- [ ] \`StreamEvent\` rows in \`orbitstream.db\` contain authentic on-chain parameters.

## Validation Steps
1. Feed a sample Soroban \`init\` event payload to \`process_event\`.
2. Inspect the resulting database row to verify non-dummy addresses.

## Tech Stack
Python 3, stellar-sdk XDR parser, SQLAlchemy"

# Issue 7: CLI
create_issue \
  "feat(cli): add cancel command for closing active streams" \
  "enhancement,difficulty: easy,good first issue,drips-wave" \
  "## Summary
\`cli/src/index.ts\` provides commands for \`init\`, \`get\`, \`claim\`, and \`approve\`, but lacks a command to invoke \`cancel_stream\` on the smart contract.

## Why It Matters
Senders using the CLI have no way to cancel active funding streams and stop token accrual without writing custom transaction scripts.

## Key Files & Modules
* \`cli/src/index.ts\` - Commander CLI definitions
* \`cli/src/bindings/stream.ts\` - Client binding interface

## Edge Cases & Constraints
* \`cancel_stream\` requires authorization from the sender (\`sender.require_auth()\`).
* CLI must check that \`SOROBAN_SECRET_KEY\` matches \`--sender\` or sign with the active keypair.

## Acceptance Criteria
- [ ] Add \`cancel\` command to \`cli/src/index.ts\`:
  \`orbitstream cancel -s <sender> -r <receiver>\`
- [ ] Loads \`SOROBAN_SECRET_KEY\` and confirms public key matches sender.
- [ ] Invokes \`client.cancel_stream({ sender, receiver })\`.
- [ ] Prints confirmation transaction hash and settled balance to stdout.
- [ ] Exits with code 1 and descriptive error message if transaction reverts.

## Validation Steps
1. Run \`npm run build\` in \`cli/\`.
2. Verify \`node dist/index.js cancel --help\` displays usage options.

## Tech Stack
TypeScript, Commander, @stellar/stellar-sdk"

# Issue 8: CLI
create_issue \
  "feat(cli): add support for escrow vault deposit and claim operations" \
  "enhancement,difficulty: medium,drips-wave" \
  "## Summary
The CLI currently only interacts with the allowance-pull Stream contract (\`contracts/stream\`). It does not support the Escrow Vault contract (\`contracts/escrow\`).

## Why It Matters
Developers using the CLI cannot test or interact with the Escrow Vault on testnet.

## Key Files & Modules
* \`cli/src/index.ts\` - CLI command structure
* \`cli/package.json\` - CLI build dependencies

## Edge Cases & Constraints
* Escrow streams are identified by a numeric \`stream_id: u64\`, unlike stream contracts which are identified by \`(sender, receiver)\`.
* \`deposit\` requires approving tokens and transferring upfront deposit capital.

## Acceptance Criteria
- [ ] Support \`ESCROW_CONTRACT_ID\` in environment configuration.
- [ ] Implement \`orbitstream escrow deposit -r <receiver> -t <token> -a <amount> -f <flow-rate>\`.
- [ ] Implement \`orbitstream escrow claim --stream-id <id>\`.
- [ ] Implement \`orbitstream escrow withdraw --stream-id <id> -a <amount>\`.
- [ ] Implement \`orbitstream escrow cancel --stream-id <id>\`.

## Validation Steps
1. \`cd cli && npm run build\` compiles without errors.
2. \`node dist/index.js escrow --help\` displays available subcommands.

## Tech Stack
TypeScript, Node.js, @stellar/stellar-sdk"

# Issue 9: Web
create_issue \
  "fix(web): handle phantom balance and insufficient allowance error states" \
  "bug,difficulty: medium,drips-wave" \
  "## Summary
In \`web/src/App.tsx\`, the UI shows a live ticker based on \`get_balance\` and \`flow_rate\`. If a sender's allowance or balance is exhausted, the ticker continues incrementing, and clicking 'Claim tokens' fails with a generic alert dialog: \`'Transaction failed on-chain.'\`.

## Why It Matters
Receivers are misled into thinking funds are available when the sender has insufficient allowance or balance. The application should proactively detect or explain this error condition.

## Key Files & Modules
* \`web/src/App.tsx\` - Claim handling and status rendering
* \`web/src/lib/soroban.ts\` - Transaction submission and error parsing
* \`web/src/components/QueryPanel.tsx\` - Query input display

## Edge Cases & Constraints
* Soroban RPC simulation errors return specific error codes or failure strings (e.g. host function invocation failure during token transfer).
* Simulation must be parsed before prompting user for signature to prevent wasted transaction fees.

## Acceptance Criteria
- [ ] Pre-simulate the claim transaction before prompting Freighter wallet.
- [ ] If simulation fails due to insufficient allowance or sender balance, set UI state to a distinct error badge (e.g. 'Sender Insufficient Allowance').
- [ ] Replace generic \`alert()\` popups with an inline error banner explaining why the claim cannot proceed.
- [ ] Freeze the live ticker from continuing to increment when insolvency is detected.

## Validation Steps
1. Test in browser with a sender having 0 allowance.
2. Confirm the dashboard displays an inline insolvency error instead of an unhandled exception.

## Tech Stack
React, TypeScript, @stellar/stellar-sdk, @stellar/freighter-api"

# Issue 10: Web
create_issue \
  "feat(web): integrate escrow vault deposit and claim flows into dashboard" \
  "enhancement,difficulty: high,drips-wave" \
  "## Summary
\`web/src/hooks/useEscrow.ts\` implements \`deposit\` and \`claim\` methods for the Escrow Vault, but it is not imported or rendered anywhere in the application. \`web/src/components/NewStreamPanel.tsx\` remains a mock labeled 'Coming soon'.

## Why It Matters
Users can only view and claim existing allowance streams; there is no UI workflow to initiate a vault stream or interact with the Escrow contract.

## Key Files & Modules
* \`web/src/hooks/useEscrow.ts\` - Escrow contract hook
* \`web/src/components/NewStreamPanel.tsx\` - Creation form component
* \`web/src/App.tsx\` - Main dashboard state container

## Edge Cases & Constraints
* Senders must have Freighter connected and sufficient token balance to cover the deposit.
* Must handle token approval step before invoking \`deposit\` on the Escrow contract.

## Acceptance Criteria
- [ ] Enable form inputs in \`NewStreamPanel.tsx\` for receiver address, token contract, deposit amount, and flow rate.
- [ ] Connect form submission to \`useEscrow().deposit\`.
- [ ] Display confirmed transaction hash with link to Stellar.Expert.
- [ ] Allow querying and claiming escrow streams by \`stream_id\`.

## Validation Steps
1. Run \`npm run build\` in \`web/\`.
2. Connect Freighter wallet and confirm deposit transaction builds and simulates successfully.

## Tech Stack
React 18, Vite, TypeScript, Tailwind CSS, @stellar/freighter-api"

# Issue 11: Web / CI
create_issue \
  "test(web): add component unit and simulation tests for stream ticker" \
  "enhancement,difficulty: medium,drips-wave" \
  "## Summary
The \`web/\` application has no automated test suite. The mathematical calculation for the live balance ticker in \`web/src/App.tsx\` (\`liveDisplay\`) and the simulation decoders in \`web/src/lib/soroban.ts\` are untested.

## Why It Matters
Regressions in bigint parsing, timestamp arithmetic, or address formatting can break the frontend without failing the CI pipeline.

## Key Files & Modules
* \`web/src/App.tsx\` - Balance interpolation and state machine
* \`web/src/lib/soroban.ts\` - RPC response unpacking
* \`web/package.json\` - Dependencies and test scripts

## Edge Cases & Constraints
* High-precision numbers: \`BigInt\` calculations must avoid floating point inaccuracies during rate interpolation.
* Timestamp deltas must handle clock skew safely.

## Acceptance Criteria
- [ ] Install Vitest and React Testing Library in \`web/\`.
- [ ] Add unit tests for \`liveDisplay\` interpolation under varying timestamps and flow rates.
- [ ] Add tests for \`formatAddress\` helper.
- [ ] Add \`npm run test\` script in \`web/package.json\` and execute it during CI in \`.github/workflows/soroban.yml\`.

## Validation Steps
1. Run \`npm run test\` in \`web/\` and confirm tests execute and pass.

## Tech Stack
TypeScript, React, Vitest"

# Issue 12: Tooling & Automation
create_issue \
  "fix(scripts): align deploy.sh and makefile with escrow and split refactor" \
  "bug,difficulty: easy,good first issue,drips-wave" \
  "## Summary
\`scripts/deploy.sh\` and \`Makefile\` still reference deploying \`target/wasm32-unknown-unknown/release/split.wasm\` (which was moved to branch \`wip/split-fanout\`). Furthermore, they do not include build or deploy targets for the newly implemented \`contracts/escrow\` contract.

## Why It Matters
Running \`make deploy\` or executing \`./scripts/deploy.sh\` will fail because \`split.wasm\` is not built on \`main\`. Additionally, developers cannot deploy the escrow vault using existing repo scripts.

## Key Files & Modules
* \`scripts/deploy.sh\` - Testnet deployment bash script
* \`Makefile\` - Root build and automation targets

## Edge Cases & Constraints
* Wasm binary name for escrow in \`Cargo.toml\` is \`orbitstream_escrow.wasm\`.
* Deployment script must save \`ESCROW_CONTRACT_ID\` to \`.env\` alongside \`STREAM_CONTRACT_ID\`.

## Acceptance Criteria
- [ ] Remove \`split.wasm\` references from \`Makefile\` and \`scripts/deploy.sh\`.
- [ ] Add build and deployment commands for \`orbitstream_escrow.wasm\`.
- [ ] Update \`deploy.sh\` to dynamically inject both \`STREAM_CONTRACT_ID\` and \`ESCROW_CONTRACT_ID\` into \`.env\`.
- [ ] Ensure \`make build-wasm\` and \`make deploy\` targets succeed cleanly on testnet.

## Validation Steps
1. Run \`make build-wasm\` and confirm both \`stream.wasm\` and \`orbitstream_escrow.wasm\` build successfully.
2. Review \`scripts/deploy.sh\` to confirm absence of \`split.wasm\` references.

## Tech Stack
Bash, Makefile, Stellar CLI"

echo "=== Issue creation script finished successfully! ==="
