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

pub async fn run_odin_deploy() -> eyre::Result<()> {
  if std::env::var("TRACE").is_ok() {
    run_deploy_with_setup(StdoutTracer::default())
  } else {
    run_deploy_with_setup(NoopTracer)
  }
}


pub async fn test_odin_host() -> eyre::Result<()> {
  let host = MockedHost::default();
  Ok(())
}

fn run_deploy_with_setup<T: Tracer>(mut tracer: T) -> eyre::Result<()> {

  let basic = Solc::new("./Basic.sol").build()?;

  let basic = basic.get("BasicCreate").expect("Could not find contract");
  let basic_bytecode = basic.bytecode.clone().to_vec();
  let basic_runtime = basic.runtime_bytecode.clone().to_vec();
  let deployer_address = "0x0fDf39DeeB7f79C9B4CD72d4e5b5DBCf072Dd929".parse().unwrap();

  let mut host = MockedHost::default();
  let deployer_account = Account {
    nonce: 0,
    code: basic_runtime.clone().into(),
    code_hash: H256::default(),
    balance: U256::from(10000000),
    storage: Default::default()
  };
  host.accounts.insert(deployer_address, deployer_account);

  let gas = 10_000_000_000;

  let sig = id("setUp()").to_vec();
  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 1,
    gas,
    destination: deployer_address,
    sender: Address::zero(),
    input_data: sig.into(),
    value: U256::zero(),
  };


  let ctr_contract = AnalyzedCode::analyze(basic_runtime);
  let accounts_before = &host.clone().accounts;

  let output = ctr_contract.execute(&mut host, &mut tracer, None, msg.clone(), Revision::latest());
  // let str =  &host.accounts.entry(callee_address).or_default().storage;
  let accounts = &host.accounts;
  println!("Addresses before:\n {:?}\n Addresses after:\n {:?}", accounts_before, accounts);
  //println!("Callee (address: {}) has storage: {:?}",callee_address, str);
  assert_eq!(output.status_code, StatusCode::Success);


  let basic_contract_addr: Address = "0xba82117bb64c0cedda9442c6e49783a9ccfbb8f6".parse().unwrap();
  let mut data = id("setVal(uint256)").to_vec();
  let mut val_to_set = U256::from(12_u64);
  let mut val_as_bytes = [0u8; 32];
  val_to_set.to_big_endian(&mut val_as_bytes);
  data.extend_from_slice(&val_as_bytes);

  let basic_contract_bytecode = host.accounts.get(&basic_contract_addr).unwrap().code.clone();

  let basic_contract = AnalyzedCode::analyze(basic_contract_bytecode.to_vec());
  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 1,
    gas,
    destination: basic_contract_addr,
    sender: Address::zero(),
    input_data: data.into(),
    value: U256::zero(),
  };

  let output = basic_contract.execute(&mut host, &mut tracer, None, msg.clone(), Revision::latest());

  let basic_contract_storage = host.accounts.get(&basic_contract_addr).unwrap();
  // Should have two storage values.
  // 0x01 => 0x01, 0x00 => 0x0c
  println!("Basic contract storage after execution: {:?}", basic_contract_storage);
  assert_eq!(output.status_code, StatusCode::Success);



  Ok(())
}
fn run_deploy<T: Tracer>(mut tracer: T) -> eyre::Result<()> {

  let basic = Solc::new("./Basic.sol").build()?;

  let basic = basic.get("BasicCreate").expect("Could not find contract");
  let basic_bytecode = basic.bytecode.clone().to_vec();
  let basic_runtime = basic.runtime_bytecode.clone().to_vec();
  let deployer_address = "0x0fDf39DeeB7f79C9B4CD72d4e5b5DBCf072Dd929".parse().unwrap();

  let mut host = MockedHost::default();
  let deployer_account = Account {
    nonce: 0,
    code: Default::default(),
    code_hash: Default::default(),
    balance: U256::from(10000000),
    storage: HashMap::new()
  };
  host.accounts.insert(deployer_address, deployer_account);

  let gas = 10_000_000_000;

  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 1,
    gas,
    destination: H160::zero(),
    sender: deployer_address,
    input_data: basic_bytecode.clone().into(),
    value: U256::from(10_000_000),
  };


  let ctr_contract = AnalyzedCode::analyze(basic_bytecode);
  let accounts_before = host.accounts.keys().collect::<Vec<&Address>>();

  let output = ctr_contract.execute(&mut host.clone(), &mut tracer, None, msg.clone(), Revision::latest());
 // let str =  &host.accounts.entry(callee_address).or_default().storage;
  let accounts = host.accounts.keys().clone().collect::<Vec<&Address>>();
  println!("Addresses before:\n {:?}\n Addresses after:\n {:?}", accounts_before, accounts);
  //println!("Callee (address: {}) has storage: {:?}",callee_address, str);
  assert_eq!(output.status_code, StatusCode::Success);



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

  assert_eq!(output.status_code, StatusCode::Success);



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

#[test]
fn test_run_deploy() {run_deploy(NoopTracer);}