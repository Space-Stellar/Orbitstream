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
