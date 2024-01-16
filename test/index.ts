import 'dotenv/config'
import {
  CasperClient, CasperServiceByJsonRPC, CLValueBuilder, Contracts,
  Keys, PurseIdentifier,
  RuntimeArgs
} from "casper-js-sdk";
import {getAccountInfo, getAccountNamedKeyValue, getBinary, getDeploy} from "./utils";
import {strict as assert} from 'assert';

const {Contract} = Contracts;
const MOTE_RATE = 1_000_000_000;

function delay(time: number) {
  return new Promise(resolve => setTimeout(resolve, time));
}

const FAUCET_KEYS = Keys.Ed25519.parseKeyFiles(
  `./accounts/faucet/public_key.pem`,
  `./accounts/faucet/secret_key.pem`
);

const USER1_KEYS = Keys.Ed25519.parseKeyFiles(
  `./accounts/user-1/public_key.pem`,
  `./accounts/user-1/secret_key.pem`
);

const USER2_KEYS = Keys.Ed25519.parseKeyFiles(
  `./accounts/user-2/public_key.pem`,
  `./accounts/user-2/secret_key.pem`
);

(async () => {
  await setup();
  await deposit();
  await testRelay();
  await testDirect();
})();

async function setup() {
  // Install sample endpoint contract
  const casperClient = new CasperClient(process.env.NODE_URL!)
  const contractClient = new Contract(casperClient);

  console.log("*** Deploy relay ***")
  const relayDeploy = contractClient.install(
    getBinary('./contracts/relay.wasm'),
    RuntimeArgs.fromMap({
      "name": CLValueBuilder.string("relay"),
    }),
    String(100 * MOTE_RATE),
    FAUCET_KEYS.publicKey,
    process.env.NETWORK_NAME!,
    [FAUCET_KEYS]
  )
  await delay(500);
  const relayHash = await relayDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, relayHash);

  let accountInfo = await getAccountInfo(process.env.NODE_URL!, FAUCET_KEYS.publicKey);
  const relayContractHash = await getAccountNamedKeyValue(
    accountInfo,
    "relay_hash");
  console.log("Relay contract hash: ", relayContractHash)

  console.log("*** Deploy sample ***")
  const sampleDeploy = contractClient.install(
    getBinary('./contracts/sample.wasm'),
    RuntimeArgs.fromMap({
      "relay_contract": CLValueBuilder.byteArray(Contracts.contractHashToByteArray(relayContractHash.slice(5))),
    }),
    String(100 * MOTE_RATE),
    FAUCET_KEYS.publicKey,
    process.env.NETWORK_NAME!,
    [FAUCET_KEYS]
  )
  await delay(500);
  const brokerHash = await sampleDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, brokerHash);

  accountInfo = await getAccountInfo(process.env.NODE_URL!, FAUCET_KEYS.publicKey);
  const sampleHash = await getAccountNamedKeyValue(
    accountInfo,
    "sample_hash");
  console.log("Sample contract hash: ", sampleHash)

  // Register contract with relay
  contractClient.setContractHash(relayContractHash);
  const registerDeploy = contractClient.callEntrypoint("register", RuntimeArgs.fromMap({
    "contract": CLValueBuilder.byteArray(Contracts.contractHashToByteArray(sampleHash.slice(5))),
    "owner": CLValueBuilder.byteArray(USER1_KEYS.publicKey.toAccountHash()),
  }), FAUCET_KEYS.publicKey, process.env.NETWORK_NAME!, String(10 * MOTE_RATE), [FAUCET_KEYS]);
  console.log("*** Register contract ***")
  await delay(500);
  const registerHash = await registerDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, registerHash);

  console.log("*** Set fee (%) ***");
  await delay(500);
  const setFeeRateDeploy = contractClient.callEntrypoint("set_fee_rate", RuntimeArgs.fromMap({
    "fee_rate": CLValueBuilder.u32(20), // 2.0%
  }), FAUCET_KEYS.publicKey, process.env.NETWORK_NAME!, String(10*MOTE_RATE), [FAUCET_KEYS]);
  const setFeeRateHash = await setFeeRateDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, setFeeRateHash);
}

async function deposit() {
  const casperClient = new CasperClient(process.env.NODE_URL!)
  const contractClient = new Contract(casperClient);
  const accountInfo = await getAccountInfo(process.env.NODE_URL!, FAUCET_KEYS.publicKey);
  const relayContractHash = await getAccountNamedKeyValue(
    accountInfo,
    "relay_hash");
  console.log("Relay contract hash: ", relayContractHash)
  contractClient.setContractHash(relayContractHash);
  const depositDeploy = contractClient.install(getBinary('./contracts/deposit.wasm'), RuntimeArgs.fromMap({
    "relay_contract": CLValueBuilder.byteArray(Contracts.contractHashToByteArray(relayContractHash.slice(5))),
    "owner": CLValueBuilder.byteArray(USER1_KEYS.publicKey.toAccountHash()),
    "amount": CLValueBuilder.u512(100*MOTE_RATE),
  }), String(10*MOTE_RATE), USER1_KEYS.publicKey, process.env.NETWORK_NAME!, [USER1_KEYS])

  const balanceBefore = await contractClient.queryContractDictionary("owner_balance", USER1_KEYS.publicKey.toAccountRawHashStr());
  console.log("Balance before: ", balanceBefore.toJSON())
  console.log('*** Deposit ***');
  await delay(500);
  const depositHash = await depositDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, depositHash);

  // Check balance
  const balanceAfter = await contractClient.queryContractDictionary("owner_balance", USER1_KEYS.publicKey.toAccountRawHashStr());
  console.log("Balance after: ", balanceAfter.toJSON())
}

async function testRelay() {
  const casperClient = new CasperClient(process.env.NODE_URL!)
  const contractClient = new Contract(casperClient);
  const accountInfo = await getAccountInfo(process.env.NODE_URL!, FAUCET_KEYS.publicKey);
  const relayContractHash = await getAccountNamedKeyValue(
    accountInfo,
    "relay_hash");
  console.log("Relay contract hash: ", relayContractHash)
  contractClient.setContractHash(relayContractHash)
  const sampleContractHash = await getAccountNamedKeyValue(
    accountInfo,
    "sample_hash");

  const setMessageDeploy = contractClient.callEntrypoint("call_on_behalf", RuntimeArgs.fromMap({
    "contract": CLValueBuilder.byteArray(Contracts.contractHashToByteArray(sampleContractHash.slice(5))),
    "entry_point": CLValueBuilder.string("set_message"),
    "caller": CLValueBuilder.byteArray(USER1_KEYS.publicKey.toAccountHash()),
    "gas_amount": CLValueBuilder.u512(10*MOTE_RATE),
    "pay_amount": CLValueBuilder.u512(0),
    "args": CLValueBuilder.byteArray(RuntimeArgs.fromMap({
      message: CLValueBuilder.string("Hello from relay")
    }).toBytes().unwrap())
  }), FAUCET_KEYS.publicKey, process.env.NETWORK_NAME!, String(10*MOTE_RATE), [FAUCET_KEYS]);

  console.log('*** Set message through relay ***');
  await delay(500);
  const setMessage = await setMessageDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, setMessage);

  contractClient.setContractHash(sampleContractHash)
  const caller = await contractClient.queryContractData(["caller"]);
  assert.equal(caller, USER1_KEYS.publicKey.toAccountRawHashStr());
  const feePurseBalance = await getFeePurseBalance(casperClient, relayContractHash);
  console.log("Fee purse balance: ", feePurseBalance.div(MOTE_RATE/100).toNumber()/100);
}

async function testDirect() {
  const casperClient = new CasperClient(process.env.NODE_URL!)
  const contractClient = new Contract(casperClient);
  const accountInfo = await getAccountInfo(process.env.NODE_URL!, FAUCET_KEYS.publicKey);
  const sampleContractHash = await getAccountNamedKeyValue(
    accountInfo,
    "sample_hash");
  console.log("Sample contract hash: ", sampleContractHash)

  contractClient.setContractHash(sampleContractHash)
  const setMessageDeploy = contractClient.callEntrypoint("set_message", RuntimeArgs.fromMap({
    "message": CLValueBuilder.string("Hello directly"),
  }), USER2_KEYS.publicKey, process.env.NETWORK_NAME!, String(10*MOTE_RATE), [USER2_KEYS]);

  console.log('*** Set message directly ***');
  await delay(500);
  const setMessage = await setMessageDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, setMessage);

  const caller = await contractClient.queryContractData(["caller"]);
  assert.equal(caller, USER2_KEYS.publicKey.toAccountRawHashStr());
}

async function getFeePurseBalance(casperClient: CasperClient, contractHash: string) {
  const rpcClient = new CasperServiceByJsonRPC(process.env.NODE_URL!);
  const rootHash = await casperClient.nodeClient.getStateRootHash();
  const blockState = await casperClient.nodeClient.getBlockState(rootHash, contractHash, []);
  const purseURef = blockState?.Contract?.namedKeys.find((item) => item.name === 'fee_purse')?.key!;
  return await rpcClient.queryBalance(PurseIdentifier.PurseUref, purseURef);
}