use rquickjs::{Context, Runtime};

pub fn list_functions(javascript: &str) -> Vec<String> {
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();
    context.with(|ctx| {
        ctx.eval::<(), _>(javascript).unwrap();
        let global = ctx.globals();
        global.keys::<String>().collect::<Result<_, _>>().unwrap()
    })
}
