import asyncio
import os
from dotenv import load_dotenv
from app.stellar.rpc_poller import SorobanEventPoller

# Load environment variables from the root .env
load_dotenv(dotenv_path='../.env')

async def main():
    rpc_url = os.getenv("SOROBAN_RPC_URL", "https://soroban-testnet.stellar.org")
    
    poller = SorobanEventPoller(rpc_url=rpc_url)
    
    try:
        await poller.poll_events()
    except KeyboardInterrupt:
        poller.stop()

if __name__ == "__main__":
    asyncio.run(main())
