use ethers::{
    abi::{self, FunctionExt},
    types::*,
    utils::{id, Solc},
};
use evm::backend::{MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::executor::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::Config;
use evm::{ExitReason, ExitRevert, ExitSucceed};
use std::collections::BTreeMap;
mod odin_vm;
use odin_vm::run_odin;
use hex::{self,encode};
use std::path::Path;
#[tokio::main]
async fn main() -> eyre::Result<()> {
    //write_code_to_file()
    //run_odin().await

    run_sputnik().await

}


fn write_code_to_file() -> eyre::Result<()> {
    let compiled = Solc::new("./CalleeTest.sol").build()?;
    let compiled = compiled.get("Callee").expect("could not find contract");
    let bytecode = compiled.runtime_bytecode.to_vec();
    let hex_code = hex::encode(bytecode);
    println!("Callee code: {}", hex_code);
    std::fs::write(Path::new("./calee.hex"), hex_code)?;


    let bytecode = compiled.bytecode.to_vec();
    let hex_code = hex::encode(bytecode);
    std::fs::write(Path::new("./callee_cons.hex"), hex_code)?;
    Ok(())
}


// Runs Callee.setGreeting -> Callee.testGreeting
async fn run_sputnik() -> eyre::Result<()> {
    let compiled = Solc::new("./CalleeTest.sol").build()?;
    let compiled_clr = Solc::new("./CallerTest.sol").build()?;
    let caller = compiled_clr.get("Caller").expect("Could not find contract");
    let callee = compiled.get("Callee").expect("Cound not find contract");


    let config = Config::istanbul();

    let vicinity = MemoryVicinity {
        gas_price: U256::zero(),
        origin: H160::default(),
        block_hashes: Vec::new(),
        block_number: Default::default(),
        block_coinbase: Default::default(),
        block_timestamp: Default::default(),
        block_difficulty: Default::default(),
        block_gas_limit: Default::default(),
        chain_id: U256::one(),
    };
    let mut state = BTreeMap::new();

    // Deploy Callee
    let callee_bytecode = callee.runtime_bytecode.clone().to_vec();

    let callee_address: Address = "0x1000000000000000000000000000000000000000"
        .parse()
        .unwrap();

    state.insert(
      callee_address,
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(10000000),
            storage: BTreeMap::new(),
            code: callee_bytecode,
        }
    );


   // Deploy Caller

    let caller_bytecode = caller.runtime_bytecode.clone().to_vec();

    let caller_address: Address = "0x2000000000000000000000000000000000000000"
        .parse()
        .unwrap();

    state.insert(
        caller_address,
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(10000000),
            storage: BTreeMap::new(),
            code: caller_bytecode,
        }
    );


    // setup memory backend w/ initial state
    let backend = MemoryBackend::new(&vicinity, state);
    let mut executor = {
        // setup gasometer
        let gas_limit = 50_000_000;
        let metadata = StackSubstateMetadata::new(gas_limit, &config);
        // setup state
        let state = MemoryStackState::new(metadata, &backend);
        // setup executor
        StackExecutor::new(state, &config)
    };

    // Make call to setGreeting()

    // let mut data = id("setGreeting(bytes32)").to_vec();
    // let mut message_set = 0x68656c6c6f_u64.to_be_bytes().to_vec();
    //
    // let mut msg_pad: Vec<u8> = vec![0;27];
    // message_set.extend_from_slice(&msg_pad);
    // let msg_slice = &message_set[3..];
    //
    //
    // data.extend_from_slice(msg_slice);
    //
    //
    // println!("Data: {:?} and data len: {:?}", data, data.len());
    //
    // // first make a call to setGreeting
    // // call the setup function
    // let from = Address::zero();
    // let to = callee_address;
    // let value = 0.into();
    // let gas_limit = 10_000_000;
    // let (reason, _) = executor.transact_call(from, to, value, data, gas_limit);
    // assert!(matches!(reason, ExitReason::Succeed(_)));
    //
    //
    //
    // // Now check expected
    //
    // let mut data = id("getExpected()").to_vec();
    //
    // let from = Address::zero();
    // let to = callee_address;
    // let value = 0.into();
    // let gas_limit = 10_000_000;
    // let (reason, ret) = executor.transact_call(from, to, value, data, gas_limit);
    // assert!(matches!(reason, ExitReason::Succeed(_)));
    // println!("Call to get expected: {:?}", ret);
    //
    // // Now retrieve the stored value for sanity
    // let data = id("greeting()").to_vec();
    // let from = Address::zero();
    // let to = callee_address;
    // let value = 0.into();
    // let gas_limit = 10_000_000;
    // let (reason, ret) = executor.transact_call(from, to, value, data, gas_limit);
    // assert!(matches!(reason, ExitReason::Succeed(_)));
    // let ret_val = ret;
    // println!("call to greeting() returned: {:?}", ret_val);
    //
    //
    // // Set callee address on caller contract

    let mut data = id("setCalleeTarget(address)").to_vec();
    let callee_addr_bytes = callee_address.to_fixed_bytes();
    data.extend_from_slice(&callee_addr_bytes);
    data.extend_from_slice(&[0; 12]);

    println!("callee address bytes: {:?}", callee_addr_bytes);
    println!("datalen: {:?} \ndata: {:?}", data.len(), data);

    let from = Address::zero();
    let to = caller_address;
    let value = 0.into();
    let gas_limit = 15_000_000;
    let (reason, ret) = executor.transact_call(from, to, value, data, gas_limit);
    println!("Reason exit on set callee: {:?}", reason);
    assert!(matches!(reason, ExitReason::Succeed(_)));
    // Run the test functions
    let test_fns = callee
        .abi
        .functions()
        .into_iter()
        .filter(|func| func.name.starts_with("test"));

    // call all the test functions
    for func in test_fns {
        // the expected result depends on the function name
        let expected = if func.name.contains("testFail") {
            ExitReason::Revert(ExitRevert::Reverted)
        } else {
            ExitReason::Succeed(ExitSucceed::Stopped)
        };

        // set the selector & execute the call
        let data = func.selector().to_vec().into();
        let (result, output) = executor.transact_call(from, to, value, data, gas_limit);

        // print the revert reason if Reverted
        if matches!(result, ExitReason::Revert(_)) {
            let revert_reason =
                abi::decode(&[abi::ParamType::String], &output[4..])?[0].to_string();
            println!("{} failed. Revert reason: \"{}\"--- Expected: {:?}", func.name, revert_reason,expected);
        }

        // ensure it worked
        assert_eq!(result, expected);
        println!("{}: {:?} -- Expected: {:?}", func.name, result, expected);
    }

    Ok(())
}