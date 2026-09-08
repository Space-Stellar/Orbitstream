import { useState, useEffect } from 'react';
import { Activity, Wallet, ArrowRightLeft, Search } from 'lucide-react';
import { isAllowed, setAllowed, getUserInfo } from '@stellar/freighter-api';
import { rpc, Address, TransactionBuilder, Networks, Operation, scValToNative } from '@stellar/stellar-sdk';

export default function App() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [senderQuery, setSenderQuery] = useState('');
  const [streamBalance, setStreamBalance] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    const checkConnection = async () => {
      try {
        if (await isAllowed()) {
          const userInfo = await getUserInfo();
          if (userInfo.publicKey) setWalletAddress(userInfo.publicKey);
        }
      } catch (e) {
        console.error("Freighter not detected");
      }
    };
    checkConnection();
  }, []);

  const connectWallet = async () => {
    try {
      let allowed = await isAllowed();
      if (!allowed) {
        await setAllowed();
        allowed = await isAllowed();
      }
      if (allowed) {
        const userInfo = await getUserInfo();
        setWalletAddress(userInfo.publicKey);
      }
    } catch (error) {
      console.error("Connection failed:", error);
    }
  };

  const checkStreamBalance = async () => {
    const contractId = import.meta.env.VITE_STREAM_CONTRACT_ID;
    if (!contractId || !walletAddress || !senderQuery) {
      alert("Please ensure your wallet is connected, the Sender Address is filled, and VITE_STREAM_CONTRACT_ID is in your web/.env");
      return;
    }

    setIsLoading(true);
    setStreamBalance(null);

    try {
      const server = new rpc.Server('https://soroban-testnet.stellar.org:443');
      // Fetch the connected account sequence to build the simulation envelope
      const account = await server.getAccount(walletAddress);

      // Build the raw XDR invocation for get_balance
      const tx = new TransactionBuilder(account, {
        fee: '1000',
        networkPassphrase: Networks.TESTNET,
      })
        .addOperation(Operation.invokeContractFunction({
          contract: contractId,
          function: 'get_balance',
          args: [
            new Address(senderQuery).toScVal(),
            new Address(walletAddress).toScVal(),
          ],
        }))
        .setTimeout(30)
        .build();

      console.log("Simulating read-only transaction on Soroban VM...");
      const simulation = await server.simulateTransaction(tx);

      if (rpc.Api.isSimulationSuccess(simulation)) {
        // Decode the binary SCVal response back into a standard JavaScript number
        const resultScVal = simulation.result.retval;
        const balance = scValToNative(resultScVal);
        setStreamBalance(balance.toString());
      } else {
        setStreamBalance("0 (Stream not found)");
      }
    } catch (error) {
      console.error("RPC Query failed:", error);
      setStreamBalance("Error querying blockchain");
    }
    
    setIsLoading(false);
  };

  const formatAddress = (address: string) => {
    if (!address) return '';
    return `${address.slice(0, 5)}...${address.slice(-4)}`;
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-4xl mx-auto space-y-8">
        
        {/* Header */}
        <header className="flex justify-between items-center bg-gray-800 p-6 rounded-xl border border-gray-700">
          <div>
            <h1 className="text-3xl font-bold flex items-center gap-3">
              <Activity className="text-blue-400" />
              OrbitStream
            </h1>
            <p className="text-gray-400 mt-1">Continuous Funding Protocol</p>
          </div>
          
          <button 
            onClick={connectWallet}
            className={`flex items-center gap-2 px-6 py-3 rounded-lg font-medium transition-colors ${
              walletAddress ? 'bg-green-600 hover:bg-green-700' : 'bg-blue-600 hover:bg-blue-700'
            }`}
          >
            <Wallet size={20} />
            {walletAddress ? formatAddress(walletAddress) : 'Connect Wallet'}
          </button>
        </header>

        {/* Dashboard Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          
          {/* Initialize Stream Card (UI Only for now) */}
          <div className="bg-gray-800 p-6 rounded-xl border border-gray-700 opacity-50">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
              <ArrowRightLeft className="text-green-400" />
              New Stream (Coming Soon)
            </h2>
            <div className="space-y-4">
              <input disabled type="text" placeholder="Receiver Address (G...)" className="w-full bg-gray-900 border border-gray-600 rounded p-3 text-white" />
              <input disabled type="number" placeholder="Flow Rate (tokens/sec)" className="w-full bg-gray-900 border border-gray-600 rounded p-3 text-white" />
              <button disabled className="w-full bg-gray-600 py-3 rounded-lg font-medium cursor-not-allowed">
                Initialize Stream
              </button>
            </div>
          </div>

          {/* Active Streams Tracker */}
          <div className="bg-gray-800 p-6 rounded-xl border border-gray-700">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
              <Search className="text-blue-400" />
              Check Incoming Stream
            </h2>
            
            <div className="space-y-4">
              <p className="text-sm text-gray-400">
                Assuming you are the receiver, enter the sender's public key to check your accrued balance.
              </p>
              
              <input 
                type="text" 
                value={senderQuery}
                onChange={(e) => setSenderQuery(e.target.value)}
                placeholder="Sender Address (G...)" 
                className="w-full bg-gray-900 border border-gray-600 rounded p-3 text-white focus:border-blue-500 focus:outline-none"
              />
              
              <button 
                onClick={checkStreamBalance}
                disabled={!walletAddress || isLoading}
                className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed py-3 rounded-lg font-medium transition-colors"
              >
                {isLoading ? 'Querying Blockchain...' : 'Check Balance'}
              </button>

              {streamBalance !== null && (
                <div className="mt-4 p-4 bg-gray-900 rounded border border-green-500/30 text-center">
                  <p className="text-gray-400 text-sm">Accrued Tokens</p>
                  <p className="text-3xl font-bold text-green-400">{streamBalance}</p>
                </div>
              )}
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
