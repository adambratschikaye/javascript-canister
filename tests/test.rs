use candid::{Decode, Encode, Principal};
use pocket_ic::{PocketIc, WasmResult};

const CANISTER_WASM: &[u8] = include_bytes!("../canister.wasm");

fn unwrap_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(contents) => contents,
        WasmResult::Reject(reject) => panic!("Message was rejected: {}", reject),
    }
}

#[test]
fn test_analyze() {
    let js = r#"
        function hello(s) {
            let result = "hello "
            return result.concat(s)
        }
        function caps(s) {
            return s.toUpperCase()
        }
        function bar() {}
    "#;
    let functions = javascript_canister::list_functions(js);
    assert_eq!(functions, vec!["hello", "caps", "bar"]);
}

#[test]
fn run_canister() {
    let js = r#"
        function hello(s) {
            let result = "hello "
            return result.concat(s)
        }

        function caps(s) {
            return s.toUpperCase()
        }

        function excited(s) {
            let result = "HELLO "
            result = result.concat(s.toUpperCase())
            return result.concat("!!")
        }
    "#;

    let names = javascript_canister::list_functions(js);
    let wasm_with_exports = javascript_canister::add_exports(&names, CANISTER_WASM);

    let pic = PocketIc::new();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    pic.install_canister(
        canister_id,
        wasm_with_exports,
        Encode!(&names, &js).unwrap(),
        None,
    );

    let result = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "hello",
            Encode!(&"foo").unwrap(),
        )
        .expect("Failed to call canister");
    let response = Decode!(&unwrap_reply(result), String).unwrap();
    assert_eq!(response, "hello foo");

    let result = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "caps",
            Encode!(&"foo").unwrap(),
        )
        .expect("Failed to call canister");
    let response = Decode!(&unwrap_reply(result), String).unwrap();
    assert_eq!(response, "FOO");

    let result = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "excited",
            Encode!(&"foo").unwrap(),
        )
        .expect("Failed to call canister");
    let response = Decode!(&unwrap_reply(result), String).unwrap();
    assert_eq!(response, "HELLO FOO!!");
}
