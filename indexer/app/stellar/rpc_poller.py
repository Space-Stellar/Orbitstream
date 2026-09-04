import aiohttp
import asyncio
import logging
import os
from sqlalchemy import select
from app.db.models import AsyncSessionLocal, SyncState, StreamEvent

logger = logging.getLogger(__name__)

class SorobanEventPoller:
    def __init__(self, rpc_url: str):
        self.rpc_url = rpc_url
        self.cursor_ledger: int = 0
        self.is_running: bool = False
        self.contract_id = os.getenv("STREAM_CONTRACT_ID")

    async def initialize_cursor(self):
        """Recover the last processed ledger from the database to prevent event loss."""
        async with AsyncSessionLocal() as session:
            result = await session.execute(select(SyncState).limit(1))
            state = result.scalar_one_or_none()
            if state:
                self.cursor_ledger = state.last_ledger
            else:
                # If no state exists, start at 0 (or a specific deployment ledger if known)
                new_state = SyncState(id=1, last_ledger=0)
                session.add(new_state)
                await session.commit()
                self.cursor_ledger = 0
        logger.info(f"Initialized poller at cursor ledger: {self.cursor_ledger}")

    async def _make_rpc_call(self, session: aiohttp.ClientSession, method: str, params: list) -> dict:
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        }
        async with session.post(self.rpc_url, json=payload) as response:
            response.raise_for_status()
            return await response.json()

    async def fetch_latest_ledger(self, session: aiohttp.ClientSession) -> int:
        data = await self._make_rpc_call(session, "getLatestLedger", [])
        if "result" in data and "id" in data["result"]:
            # Soroban RPC returns the sequence as 'id' or 'sequence' depending on the version
            return data["result"].get("sequence", data["result"].get("id", self.cursor_ledger))
        return self.cursor_ledger
