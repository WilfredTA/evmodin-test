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

use std::collections::BTreeMap;
use evmodin::util::mocked_host::Account;
use std::str::FromStr;

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
  let compiled = Solc::new("./*.sol").build()?;
  let compiled = compiled.get("Greet").expect("could not find contract");
  let bytecode = compiled.runtime_bytecode.clone().to_vec();


  let contract = AnalyzedCode::analyze(bytecode);

  let mut host = MockedHost::default();

  let setup_id = id("setUp()").to_vec();
  let gas = 10_000_000_000;

  let msg = Message {
    kind: CallKind::Call,
    is_static: false,
    depth: 1,
    gas,
    destination: Address::zero(),
    sender: Address::zero(),
    input_data: setup_id.into(),
    value: U256::zero(),
  };


  let output = contract.execute(&mut host, &mut tracer, None, msg.clone(), Revision::latest());
  let str =  &host.accounts.entry(Address::zero()).or_default().storage;
  println!("Storage is: {:?}", str);
  assert_eq!(output.status_code, StatusCode::Success);

  let test_fns = compiled
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

    let output = contract.execute(&mut host, &mut tracer, None, msg, Revision::latest());

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