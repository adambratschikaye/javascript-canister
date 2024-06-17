use candid::{encode_one, Decode, Principal};
use pocket_ic::{PocketIc, WasmResult};

fn unwrap_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(contents) => contents,
        WasmResult::Reject(reject) => panic!("Message was rejected: {}", reject),
    }
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

    let wasm = javascript_canister::build(js);

    let pic = PocketIc::new();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    pic.install_canister(canister_id, wasm, encode_one(()).unwrap(), None);

    let result = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "hello",
            encode_one("foo").unwrap(),
        )
        .expect("Failed to call canister");
    let response = Decode!(&unwrap_reply(result), String).unwrap();
    assert_eq!(response, "hello foo");

    let result = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "caps",
            encode_one("foo").unwrap(),
        )
        .expect("Failed to call canister");
    let response = Decode!(&unwrap_reply(result), String).unwrap();
    assert_eq!(response, "FOO");

    let result = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "excited",
            encode_one("foo").unwrap(),
        )
        .expect("Failed to call canister");
    let response = Decode!(&unwrap_reply(result), String).unwrap();
    assert_eq!(response, "HELLO FOO!!");
}
