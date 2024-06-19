use candid::{encode_one, Decode};
use rquickjs::{Context, Function, Runtime};
use std::cell::RefCell;

thread_local! {
    static JS: RefCell<Option<(Vec<String>, Runtime, Context)>> = RefCell::new(None);
}

#[export_name = "wizer.initialize"]
fn pre_initialize() {
    let js = std::fs::read_to_string("code.js").unwrap();
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();
    let names: Vec<String> = context.with(|ctx| {
        ctx.eval::<(), _>(js).unwrap();
        let global = ctx.globals();
        global.keys::<String>().collect::<Result<_, _>>().unwrap()
    });
    std::fs::write("names.txt", names.join("\n")).unwrap();
    JS.with(|js| {
        *js.borrow_mut() = Some((names, runtime, context));
    })
}

#[ic_cdk::init]
fn init() {
    ic_wasi_polyfill::init(&[0u8; 32], &[]);
}

#[no_mangle]
fn call_js(index: usize) {
    JS.with(|js| {
        let mut js = js.borrow_mut();
        if js.is_none() {
            panic!("No JS loaded");
        }
        let arg = ic_cdk::api::call::arg_data_raw();
        let js_arg = Decode!(&arg, String).unwrap();
        let (names, _, context) = js.as_mut().unwrap();
        let js_result: String = context.with(|ctx| {
            let name = &names[index];
            let function: Function = ctx.globals().get(name).unwrap();
            let result: String = function.call((js_arg,)).unwrap();
            result
        });

        let response = encode_one(js_result).unwrap();
        ic_cdk::api::call::reply_raw(&response);
    })
}
