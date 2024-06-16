use candid::{Decode, Encode, Principal};
use pocket_ic::{PocketIc, WasmResult};

const CANISTER_WASM: &[u8] = include_bytes!("../target/wasm32-wasi/release/canister.wasm");

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
    let pre_initialized = javascript_canister::pre_initialize(CANISTER_WASM, js, &names);
    let wasm_with_exports = javascript_canister::add_exports(&names, &pre_initialized);
    let wasi_path = "canister_wasi.wasm";
    let ic_path = "canister.wasm";
    std::fs::write(wasi_path, wasm_with_exports).unwrap();
    javascript_canister::run_wasi2ic(wasi_path, ic_path);
    let final_wasm = std::fs::read(ic_path).unwrap();

    let pic = PocketIc::new();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    pic.install_canister(canister_id, final_wasm, Encode!(&()).unwrap(), None);

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
