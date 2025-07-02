import { ChainConfig, Client } from 'alith/lazai'
import { Agent } from 'alith'

async function main() {
  const node = '0x34d9E02F9bB4E4C8836e38DF4320D4a79106F194'
  const client = new Client(ChainConfig.testnet())
  // Add by the inference node admin
  // await client.addInferenceNode(node, "http://127.0.0.1:8000", "<Replace with your RSA pub key>")
  try {
    // Note: init only once, catch the UserAlreadyExists errors
    await client.addUser(100_000_000)
    await client.depositInference(node, 50_000_000)
  } catch {}
  console.log('The inference account of user is', await client.getInferenceAccount(client.getWallet().address, node))
  const fileId = 1
  const nodeInfo = await client.getInferenceNode(node)
  const url = nodeInfo.url
  const agent = new Agent({
    // Note: replace with your model here
    model: 'Qwen-2.5',
    // OpenAI-compatible inference server URL
    baseUrl: `${url}/v1`,
    // Extra headers for settlement and DAT file anchoring
    extraHeaders: await client.getRequestHeaders(node, BigInt(fileId)),
  })
  console.log(await agent.prompt('What is Alith?'))
}

await main()
