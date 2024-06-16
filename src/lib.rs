use std::process::Command;

use rquickjs::{Context, Runtime};
use walrus::{
    ir::{Call, Const, Instr, Value},
    ExportItem, FunctionBuilder, ModuleConfig,
};
use wizer::Wizer;

pub fn list_functions(javascript: &str) -> Vec<String> {
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();
    context.with(|ctx| {
        ctx.eval::<(), _>(javascript).unwrap();
        let global = ctx.globals();
        global.keys::<String>().collect::<Result<_, _>>().unwrap()
    })
}

pub fn pre_initialize(wasm: &[u8], js: &str, names: &[String]) -> Vec<u8> {
    std::fs::write("names.txt", names.join("\n")).unwrap();
    std::fs::write("code.js", js).unwrap();
    let mut wizer = Wizer::new();
    wizer.wasm_bulk_memory(true);
    wizer.allow_wasi(true).unwrap();
    wizer.dir(".");
    wizer.run(wasm).unwrap()
}

pub fn add_exports(names: &[String], wasm: &[u8]) -> Vec<u8> {
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

pub fn run_wasi2ic(src_path: &str, output_path: &str) {
    let mut command = Command::new("wasi2ic");
    command.arg(src_path).arg(output_path);
    command.output().unwrap();
}
