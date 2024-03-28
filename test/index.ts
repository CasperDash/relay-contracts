import 'dotenv/config'
import {
  CasperClient, CLValueBuilder, Contracts,
  Keys, RuntimeArgs
} from "casper-js-sdk";
import {getAccountInfo, getAccountNamedKeyValue, getBinary, getDeploy} from "./utils";

const {Contract} = Contracts;
const MOTE_RATE = 1_000_000_000;

function delay(time: number) {
  return new Promise(resolve => setTimeout(resolve, time));
}

const ADMIN_KEYS = Keys.Ed25519.parseKeyFiles(
  `./accounts/admin/public_key.pem`,
  `./accounts/admin/secret_key.pem`
);

(async () => {
  await setup();
  await deposit();
})();

async function setup() {
  // Install relay contract
  const casperClient = new CasperClient(process.env.NODE_URL!)
  const contractClient = new Contract(casperClient);

  console.log("*** Deploy relay ***")
  const relayDeploy = contractClient.install(
    getBinary('./contracts/relay.wasm'),
    RuntimeArgs.fromMap({
      "name": CLValueBuilder.string("relay"),
    }),
    String(200 * MOTE_RATE),
    ADMIN_KEYS.publicKey,
    process.env.NETWORK_NAME!,
    [ADMIN_KEYS]
  )
  await delay(500);
  const relayHash = await relayDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, relayHash);

  let accountInfo = await getAccountInfo(process.env.NODE_URL!, ADMIN_KEYS.publicKey);
  const relayContractHash = await getAccountNamedKeyValue(
    accountInfo,
    "relay_hash");
  console.log("Relay contract hash: ", relayContractHash)
  const relayContractPackageHash = await getAccountNamedKeyValue(
    accountInfo,
    "relay_package_name");
  console.log("Relay contract package hash: ", relayContractPackageHash)

  // Register contract with relay
  contractClient.setContractHash(relayContractHash);
  const registerDeploy = contractClient.callEntrypoint("register", RuntimeArgs.fromMap({
    "contract": CLValueBuilder.byteArray(Contracts.contractHashToByteArray(process.env.CEP78_CONTRACT_HASH!)),
    "owner": CLValueBuilder.byteArray(ADMIN_KEYS.publicKey.toAccountHash()),
  }), ADMIN_KEYS.publicKey, process.env.NETWORK_NAME!, String(10 * MOTE_RATE), [ADMIN_KEYS]);
  console.log("*** Register contract ***")
  await delay(500);
  const registerHash = await registerDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, registerHash);
}

async function deposit() {
  const casperClient = new CasperClient(process.env.NODE_URL!)
  const contractClient = new Contract(casperClient);
  const accountInfo = await getAccountInfo(process.env.NODE_URL!, ADMIN_KEYS.publicKey);
  const relayContractHash = await getAccountNamedKeyValue(
    accountInfo,
    "relay_hash");
  console.log("Relay contract hash: ", relayContractHash)
  contractClient.setContractHash(relayContractHash);
  const depositDeploy = contractClient.install(getBinary('./contracts/deposit.wasm'), RuntimeArgs.fromMap({
    "relay_contract": CLValueBuilder.byteArray(Contracts.contractHashToByteArray(relayContractHash.slice(5))),
    "owner": CLValueBuilder.byteArray(ADMIN_KEYS.publicKey.toAccountHash()),
    "amount": CLValueBuilder.u512(100 * MOTE_RATE),
  }), String(10 * MOTE_RATE), ADMIN_KEYS.publicKey, process.env.NETWORK_NAME!, [ADMIN_KEYS])

  console.log('*** Deposit ***');
  await delay(500);
  const depositHash = await depositDeploy.send(process.env.NODE_URL!);
  await getDeploy(process.env.NODE_URL!, depositHash);
}