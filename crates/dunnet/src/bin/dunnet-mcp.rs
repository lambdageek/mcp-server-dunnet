use dunnet;
use tokio::runtime;

pub fn main() {
    let rt = runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        let mut _dn = dunnet::Dunnet::new();
        // dunnet mcp repl
        _dn.mcp_server().await;
    })
}
