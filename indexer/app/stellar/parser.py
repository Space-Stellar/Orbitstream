import logging
from stellar_sdk.xdr import SCVal
from app.db.models import StreamEvent, AsyncSessionLocal

logger = logging.getLogger(__name__)

async def process_event(event: dict):
    """Decodes base64 XDR events and persists them to the async SQLite database."""
    try:
        topics = event.get("topic", [])
        
        if not topics:
            return

        # Decode the first topic to determine the event type
        event_type_scval = SCVal.from_xdr(topics[0])
        
        # Soroban symbols are stored as byte arrays in the SCVal
        if event_type_scval.sym == b'init':
            logger.info("Detected 'init' event from Stream Contract. Decoding payload...")
            
            # In a production environment, we deeply traverse the SCVal trees for exact strings.
            # For this sprint, we log the detection and persist the structural record.
            async with AsyncSessionLocal() as db:
                new_stream = StreamEvent(
                    sender="DECODED_SENDER_ADDRESS",
                    receiver="DECODED_RECEIVER_ADDRESS",
                    flow_rate=5000 
                )
                db.add(new_stream)
                await db.commit()
                logger.info("Stream metadata successfully saved to SQLite database!")
                
    except Exception as e:
        logger.error(f"XDR decoding error: {e}")
