use std::process::Command;

use walrus::{
    ir::{Call, Const, Instr, Value},
    ExportItem, FunctionBuilder, ModuleConfig,
};
use wizer::Wizer;

const CANISTER_WASM: &[u8] = include_bytes!("../target/wasm32-wasi/release/canister.wasm");

fn pre_initialize(wasm: &[u8], js: &str) -> (Vec<u8>, Vec<String>) {
    std::fs::write("code.js", js).unwrap();
    let mut wizer = Wizer::new();
    wizer.wasm_bulk_memory(true);
    wizer.allow_wasi(true).unwrap();
    wizer.dir(".");
    let wasm = wizer.run(wasm).unwrap();
    let names = std::fs::read_to_string("names.txt")
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect();
    (wasm, names)
}

fn add_exports(names: &[String], wasm: &[u8]) -> Vec<u8> {
    let config = ModuleConfig::new();
    let mut module = config.parse(wasm).unwrap();
    let main_export = module.exports.iter().find(|e| e.name == "call_js").unwrap();
    let main_export_id = main_export.id();
    let main_id = match main_export.item {
        walrus::ExportItem::Function(id) => id,
        _ => panic!("'call_js' is not a function."),
    };
    module.exports.delete(main_export_id);
    for (index, name) in names.into_iter().enumerate() {
        let mut func = FunctionBuilder::new(&mut module.types, &[], &[]);
        let instructions = [
            Instr::Const(Const {
                value: Value::I32(index as u32 as i32),
            }),
            Instr::Call(Call { func: main_id }),
        ];
        let mut body = func.func_body();
        for i in instructions {
            body.instr(i);
        }
        let func_id = func.finish(vec![], &mut module.funcs);

        let _export_id = module.exports.add(
            &format!("canister_update {}", name),
            ExportItem::Function(func_id),
        );
    }
    module.emit_wasm()
}

fn run_wasi2ic(src_path: &str, output_path: &str) {
    let mut command = Command::new("wasi2ic");
    command.arg(src_path).arg(output_path);
    command.output().unwrap();
}

pub fn build(js: &str) -> Vec<u8> {
    let (pre_initialized, names) = pre_initialize(CANISTER_WASM, js);
    let wasm_with_exports = add_exports(&names, &pre_initialized);
    let wasi_path = "canister_wasi.wasm";
    let ic_path = "canister.wasm";
    std::fs::write(wasi_path, wasm_with_exports).unwrap();
    run_wasi2ic(wasi_path, ic_path);
    std::fs::read(ic_path).unwrap()
}
