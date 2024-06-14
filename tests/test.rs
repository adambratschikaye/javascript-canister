// use candid::{encode_one, Decode, Principal};
// use pocket_ic::{PocketIc, WasmResult};

// const CANISTER_WASM: &[u8] =
//     include_bytes!("../target/wasm32-unknown-unknown/release/canister.wasm");

// #[test]
// fn test_hello() {
//     let pic = PocketIc::new();
//     // Create an empty canister as the anonymous principal and add cycles.
//     let canister_id = pic.create_canister();
//     pic.add_cycles(canister_id, 2_000_000_000_000);

//     pic.install_canister(canister_id, CANISTER_WASM.to_vec(), vec![], None);
//     let result = pic
//         .update_call(
//             canister_id,
//             Principal::anonymous(),
//             "hello",
//             encode_one("world").unwrap(),
//         )
//         .expect("Failed to call canister");

//     let reply = Decode!(&unwrap_reply(result), String).unwrap();
//     assert_eq!(reply, "hello world");
// }

// fn unwrap_reply(result: WasmResult) -> Vec<u8> {
//     match result {
//         WasmResult::Reply(contents) => contents,
//         WasmResult::Reject(reject) => panic!("Message was rejected: {}", reject),
//     }
// }

#[test]
fn test_analyze() {
    let js = r#"
        function hello(s) {
            result = "hello "
            return result.concat(s)
        }
        function caps(s) {
            return s.toUpperCase()
        }
    "#;
    let functions = javascript_canister::list_functions(js);
    assert_eq!(functions, vec!["hello", "caps"]);
}
