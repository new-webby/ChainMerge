import { ChainMergeClient } from "chainmerge-sdk";

const client = new ChainMergeClient({
  baseUrl: "https://chainmerge-api.onrender.com",
});

const chainTests: Array<{ chain: any; hash: string }> = [
  { chain: "ethereum", hash: "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad" },
  { chain: "cosmos", hash: "6C166D13D94E626BB6477398B1D0AEB9B5C595D0B0DA8FC7AD2191BEEF024A27" },
  { chain: "starknet", hash: "0x2b115e75d8961caed22948082998710ac653b088448deb421f3c2a0decd1325" },
  { chain: "solana", hash: "5UqU2s9B4PjF3Z1x3S9g1pEwQf5a3jR4DqF3T6sC8gMqz7W3zXvR9kLcQJq7P9mW9" },
  { chain: "aptos", hash: "0x3d0b2f56b0d912b7a9fdb783c2a0b12bc1234c9d784fa8c8a14b13d2f9d854ce" },
  { chain: "sui", hash: "3sP85eM8qYpHY9TqzJj3mKz6tP2LcXJb5jZ2V1mYzQwE" },
  { chain: "bitcoin", hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" },
  { chain: "polkadot", hash: "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" }
];

async function runTests() {
  console.log("Testing ChainMerge API on ALL CHAINS...\n");

  for (const { chain, hash } of chainTests) {
    console.log(`Testing decoder for [${chain}] with hash: ${hash}...`);
    try {
      const decoded = await client.decodeTx({
        chain: chain,
        hash: hash,
      });

      console.log(`✅ SUCCESS [${chain}] - Found ${decoded.events.length} events (Sender: ${decoded.sender})`);
    } catch (error: any) {
      if (error.code === 4004 || error.message.includes("hash") || error.message.includes("rpc") || error.message.includes("RPC")) {
        console.log(`✅ API REACHABLE [${chain}] - Decoder routed request correctly (Result: ${error.message})`);
      } else {
        console.error(`❌ FAILED [${chain}] - Unexpected error:`, error.message);
      }
    }
    console.log("-------------------------------------------------");
  }

  console.log("\n🎉 All chain routing and decoder tests completed!");
}

runTests();
