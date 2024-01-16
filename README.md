# Relay contract

## relay
Main contract for relay
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

## deposit
Session contract (WASM) to deposit CSPR to pay for gas
- `owner`: which owner account to deposit to
- `amount`: amount of CSPR to deposit

## test
E2E test for CBRC20 Marketplace

#### Setup
- Create a `.env` file for network configuration
- Copy keys to `accounts` folder
- `npm install`
- run `index.ts` 