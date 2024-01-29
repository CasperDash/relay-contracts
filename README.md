# Relay contract

## relay
Main contract for relay 
>Testnet address: `6112c4d26e0b91bc2cb838cde4da41e9fc706b98d60225ce2070ca4eb9d69fbf`

#### *Entrypoint* call_on_behalf
Call a contract on behalf of a user
- `contract`: recipient contract hash
- `entry_point`: recipient contract entry point
- `caller`: actual caller
- `gas_amount`: gas amount
- `pay_amount`: payment amount
- `args`: recipient contract arguments

#### *Entrypoint* register
Register a contract to be able to call from relay
- `contract`: contract hash
- `owner`: contract owner

## sample
Sample contract to test relay 
>Testnet address: `2f17ce27d18c5aa1129e9cf6a3f7cb9680ff0703bc6d9751a079c49c482b638a`

## deposit
Session contract (WASM) to deposit CSPR to pay for gas
- `owner`: which owner account to deposit to
- `amount`: amount of CSPR to deposit

## test
E2E test for relay contract

#### Setup
- Create a `.env` file for network configuration
- Copy keys to `accounts` folder
- `npm install`
- run `index.ts` 