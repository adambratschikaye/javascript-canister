#[ic_cdk::query]
fn hello(name: String) -> String {
    format!("hello {}", name)
}
