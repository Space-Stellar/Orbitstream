from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker, AsyncAttrs
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column
from sqlalchemy import Integer, String, BigInteger

class Base(AsyncAttrs, DeclarativeBase):
    pass

class SyncState(Base):
    """Tracks the last processed ledger to prevent event dropping upon restart."""
    __tablename__ = 'sync_state'
    id: Mapped[int] = mapped_column(primary_key=True)
    last_ledger: Mapped[int] = mapped_column(Integer, default=0)

class StreamEvent(Base):
    """Stores on-chain stream initialization events."""
    __tablename__ = 'stream_events'
    id: Mapped[int] = mapped_column(primary_key=True, autoincrement=True)
    sender: Mapped[str] = mapped_column(String(56))
    receiver: Mapped[str] = mapped_column(String(56))
    flow_rate: Mapped[int] = mapped_column(BigInteger)

# Initialize the async SQLite engine
engine = create_async_engine("sqlite+aiosqlite:///orbitstream.db", echo=False)
AsyncSessionLocal = async_sessionmaker(engine, expire_on_commit=False)

async def init_db():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
