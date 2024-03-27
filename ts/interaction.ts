import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import fs from "fs";

// Define the sender's private key
// const privateKey = "your_private_key_here";

// Create a signer object using the private key
// const wallet = await DirectSecp256k1Wallet.fromKey(privateKey);
// const mnemonic = 

// Create a wallet from the mnemonic
const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: "neutron",
});

// Initialize a CosmWasm client with the signer
const client = await SigningCosmWasmClient.connectWithSigner("https://rpc-palvus.pion-1.ntrn.tech", wallet, {
    gasPrice: GasPrice.fromString("0.025untrn"),
});

// Define the sender's address and the contract address
const [account] = await wallet.getAccounts();
const senderAddress = account.address;
console.log(senderAddress)

// deploy

const wasm= fs.readFileSync("/cw-subscription/artifacts/cw_subscription.wasm")
const result = await client.upload(senderAddress, wasm, "auto")
console.log(result)

// instantiate

const codeId = result.codeId; // 

//Define the instantiate message
const instantiateMsg = { "admin": senderAddress }; // for the staking contract

//Instantiate the contract
const instantiateResponse = await client.instantiate(senderAddress, codeId, instantiateMsg, "Subscription", "auto")
console.log(instantiateResponse)

const contractAddress = instantiateResponse.contractAddress

const createPlanResult1 = await client.execute(senderAddress, contractAddress, {create_plan: {name: "Plan", description: "Description", price: "10000", freeze_right_per_subscriber: 10, frequency: 15}}, "auto")

const queryPlanById = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})
console.log(queryPlanById)

const subscribeResult = await client.execute(senderAddress, contractAddress, {subscribe: {id: 1}}, "auto", "", [{denom: "untrn", amount: "10000"}])
const query_subscriber = await client.queryContractSmart(contractAddress, {query_subscriber: {id: 1, address: senderAddress}})
console.log(query_subscriber)

const queryPlanById2 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})
console.log(queryPlanById2)

const paySubscriptionResult = await client.execute(senderAddress, contractAddress, {pay_subscription: {id: 1}}, "auto", "", [{denom: "untrn", amount: "10000"}])
const query_subscriber4 = await client.queryContractSmart(contractAddress, {query_subscriber: {id: 1, address: senderAddress}})
console.log(query_subscriber4)

const queryPlanById3 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})
console.log(queryPlanById3)

const freezeSubscription = await client.execute(senderAddress, contractAddress, {freeze_subscription: {id: 1, duration_day: 5}}, "auto") 
const query_subscriber5 = await client.queryContractSmart(contractAddress, {query_subscriber: {id: 1, address: senderAddress}})
console.log(query_subscriber5)

const queryPlanById4 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})
console.log(queryPlanById4)

const unsubscribeResult = await client.execute(senderAddress, contractAddress, {cancel_subscription: {id: 1}}, "auto")
const query_subscriber6 = await client.queryContractSmart(contractAddress, {query_subscriber: {id: 1, address: senderAddress}})
console.log(query_subscriber6)

const queryPlanById5 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})
console.log(queryPlanById5)

const renewSubscription = await client.execute(senderAddress, contractAddress, {renew_subscription: {id: 1}}, "auto", "", [{denom: "untrn", amount: "10000"}])
const query_subscriber7 = await client.queryContractSmart(contractAddress, {query_subscriber: {id: 1, address: senderAddress}})
console.log(query_subscriber7)

const queryPlanById6 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})
console.log(queryPlanById6)

const updatePlanResult = await client.execute(senderAddress, contractAddress, {update_plan: {id: 1, name: "Plan", description: "Description", price: "10000", freeze_right_per_subscriber: 10, frequency: 15}}, "auto")
const queryPlanById10 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})

const removePlanResult = await client.execute(senderAddress, contractAddress, {remove_plan: {id: 1}}, "auto" );

const queryPlanById7 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 1}})
console.log(queryPlanById7)

const createPlanResult2 = await client.execute(senderAddress, contractAddress, {create_plan: {name: "Plan 2", description: "Description 2", price: "20000", freeze_right: 10, frequency: 15}}, "auto")
const queryPlanById9 = await client.queryContractSmart(contractAddress, {query_plan_by_id: {id: 2}})

const subscribeResult2 = await client.execute(senderAddress, contractAddress, {subscribe: {id: 2}}, "auto", "", [{denom: "untrn", amount: "20000"}])


const query_subscriber8 = await client.queryContractSmart(contractAddress, {query_subscriber: {id: 2, address: senderAddress}})


const withdrawResult2 = await client.execute(senderAddress, contractAddress, {withdraw_payments: {id: 1, amount: "20000"}}, "auto")