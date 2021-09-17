use evmodin::{
  tracing::{NoopTracer, StdoutTracer, Tracer},
  util::mocked_host::MockedHost,
  AnalyzedCode, CallKind, Message, Revision, StatusCode,
};

use ethers::{
  abi::{self, FunctionExt},
  types::*,
  utils::{id, Solc},
};

use std::collections::{BTreeMap, HashMap};
use evmodin::util::mocked_host::Account;
use std::str::FromStr;
use std::convert::{Into, From};

pub async fn run_odin() -> eyre::Result<()> {
  if std::env::var("TRACE").is_ok() {
    run(StdoutTracer::default())
  } else {
    run(NoopTracer)
  }
}

pub async fn test_odin_host() -> eyre::Result<()> {
  let host = MockedHost::default();
  Ok(())
}

fn run<T: Tracer>(mut tracer: T) -> eyre::Result<()> {
  let compiled_callee = Solc::new("./CalleeTest.sol").build()?;
  let compiled_caller = Solc::new("./CallerTest.sol").build()?;

  let callee_compiled = compiled_callee.get("Callee").expect("could not find contract");
  let caller_compiled = compiled_caller.get("Caller").expect("could not find contract");
  let callee = callee_compiled.runtime_bytecode.clone().to_vec();
  let caller =caller_compiled.runtime_bytecode.clone().to_vec();


  let callee_acc = Account {
    nonce: 0,
    code: callee.clone().into(),
    code_hash: H256::random(),
    balance: U256::from(10000000),
    storage: HashMap::new()

  };
  let caller_acc = Account {
    nonce: 0,
    code: caller.clone().into(),
    code_hash: H256::default(),
    balance: U256::from(10000000),
    storage: HashMap::new()

  };
  let callee_address = "0x38db92b540acf663d72611e8c7c3cf219c7962e0".parse().unwrap();

  let caller_address = "0xb685955fb84725e230587ebdb1d8461d2e09783d".parse().unwrap();

  let mut host = MockedHost::default();
  host.accounts.insert(callee_address, callee_acc);
  host.accounts.insert(caller_address, caller_acc);

  let callee_contract = AnalyzedCode::analyze(callee);

  let caller_contract = AnalyzedCode::analyze(caller);
  let gas = 10_000_000_000;
  //
  //Set Greeting in Callee
  let mut data = id("setGreeting(bytes32)").to_vec();
  let mut message_set = 0x68656c6c6f_u64.to_be_bytes().to_vec();

  let mut msg_pad: Vec<u8> = vec![0;27];
  message_set.extend_from_slice(&msg_pad);
  let msg_slice = &message_set[3..];


  data.extend_from_slice(msg_slice);



  //
  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 1,
    gas,
    destination: callee_address,
    sender: Address::zero(),
    input_data: data.into(),
    value: U256::zero(),
  };


  let output = callee_contract.execute(&mut host, &mut tracer, None, msg.clone(), Revision::latest());
  let str =  &host.accounts.entry(callee_address).or_default().storage;
  //println!("Callee (address: {}) has storage: {:?}",callee_address, str);
  assert_eq!(output.status_code, StatusCode::Success);

  let mut data = id("setCalleeTarget(address)").to_vec();
  let callee_addr_bytes = callee_address.to_fixed_bytes();
  data.extend_from_slice(&[0; 12]);
  data.extend_from_slice(&callee_addr_bytes);


  let sender = Address::zero();
  let destination = caller_address;
  let value = U256::zero();


  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 1,
    gas,
    destination,
    sender,
    input_data: data.into(),
    value
  };

  let output = caller_contract.execute(&mut host, &mut tracer, None, msg.clone(), Revision::latest());
  let str =  &host.accounts.entry(caller_address).or_default().storage;
  //println!("Caller's (address: {}) storage is: {:?}",caller_address, str);
  assert_eq!(output.status_code, StatusCode::Success);

  let mut data = id("callCalleeSetGreeting()").to_vec();

  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 1,
    gas,
    destination,
    sender,
    input_data: data.clone().into(),
    value
  };

  let output = caller_contract.execute(&mut host, &mut tracer, None, msg.clone(), Revision::latest());
  let str =  &host.accounts.entry(callee_address).or_default().storage;
  //println!("Callee's (address: {}) storage is: {:?}",callee_address, str);
  //println!("Recorded Calls: {:?}", host.recorded);
  assert_eq!(output.status_code, StatusCode::Success);
  //


  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 0,
    gas,
    destination: callee_address,
    sender,
    input_data: Default::default(),
    value
  };


  // Tests
  let str =  &host.accounts.entry(callee_address).or_default().storage;
  //println!("Callee's (address: {}) storage is: {:?}",callee_address, str);
  let test_fns = callee_compiled
      .abi
      .functions()
      .into_iter()
      .filter(|func| func.name.starts_with("test"));

  for func in test_fns {
    let expected = if func.name.contains("testFail") {
      StatusCode::Revert
    } else {
      StatusCode::Success
    };


    let mut msg = msg.clone();

    msg.input_data = func.selector().to_vec().into();

    let output = callee_contract.execute(&mut host, &mut tracer, None, msg, Revision::latest());
    let target_storage =  &host.accounts.entry(callee_address).or_default().storage;
    println!("Storage after calls: {:?}", target_storage);
    if output.status_code == StatusCode::Revert {

      let revert_reason = abi::decode(&[abi::ParamType::String],
      &output.output_data[4..])?[0].to_string();
      println!("{} failed. Revert reason: \"{}\"", func.name, revert_reason);
    }

    assert_eq!(output.status_code, expected);
    println!("{}: {:?}", func.name, output.status_code);
  }

  Ok(())

}

#[test]
fn test_run() {
  run(NoopTracer);
}