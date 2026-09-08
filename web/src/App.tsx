import { useState, useEffect } from 'react';
import { Activity, Wallet, ArrowRightLeft } from 'lucide-react';
import { isAllowed, setAllowed, getUserInfo } from '@stellar/freighter-api';

export default function App() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);

  useEffect(() => {
    // Check if the user previously connected their wallet
    const checkConnection = async () => {
      try {
        if (await isAllowed()) {
          const userInfo = await getUserInfo();
          if (userInfo.publicKey) setWalletAddress(userInfo.publicKey);
        }
      } catch (e) {
        console.error("Freighter not detected or locked");
      }
    };
    checkConnection();
  }, []);

  const connectWallet = async () => {
    try {
      let allowed = await isAllowed();
      if (!allowed) {
        await setAllowed(); // Prompts the user to approve the connection
        allowed = await isAllowed();
      }
      
      if (allowed) {
        const userInfo = await getUserInfo();
        setWalletAddress(userInfo.publicKey);
      }
    } catch (error) {
      console.error("Connection failed:", error);
      alert("Please ensure the Freighter wallet extension is installed and unlocked.");
    }
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
          
          {/* Initialize Stream Card */}
          <div className="bg-gray-800 p-6 rounded-xl border border-gray-700">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
              <ArrowRightLeft className="text-green-400" />
              New Stream
            </h2>
            <div className="space-y-4">
              <input 
                type="text" 
                placeholder="Receiver Address (G...)" 
                className="w-full bg-gray-900 border border-gray-600 rounded p-3 text-white focus:border-blue-500 focus:outline-none"
              />
              <input 
                type="number" 
                placeholder="Flow Rate (tokens/sec)" 
                className="w-full bg-gray-900 border border-gray-600 rounded p-3 text-white focus:border-blue-500 focus:outline-none"
              />
              <button 
                disabled={!walletAddress}
                className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed py-3 rounded-lg font-medium transition-colors"
              >
                {walletAddress ? 'Initialize Stream' : 'Connect Wallet First'}
              </button>
            </div>
          </div>

          {/* Active Streams Card */}
          <div className="bg-gray-800 p-6 rounded-xl border border-gray-700">
            <h2 className="text-xl font-semibold mb-4">Active Streams</h2>
            <div className="bg-gray-900 p-8 rounded border border-gray-600 text-center text-gray-500">
              {walletAddress 
                ? "No active streams found for this address."
                : "Connect your wallet to view active streams and claim balances."}
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
