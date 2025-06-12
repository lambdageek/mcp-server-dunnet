
use dunnet;
use tokio::runtime;

pub fn main() {
    let rt = runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on (async {
        let mut dn = dunnet::Dunnet::new();
        dn.repl().await;
    })
}