use async_trait::async_trait;
use rust_mcp_sdk::schema::{
    CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, RpcError,
    schema_utils::CallToolError,
};
use rust_mcp_sdk::{McpServer, mcp_server::ServerHandler};

use super::tools::DunnetTools;
use crate::DunnetRepl;

// Custom Handler to handle MCP Messages
pub struct DunnetHandler {
    repl: DunnetRepl,
}

impl DunnetHandler {
    /// Creates a new instance of `DunnetHandler` with the provided `DunnetRepl`.
    pub fn new(repl: DunnetRepl) -> Self {
        DunnetHandler { repl }
    }
}

// To check out a list of all the methods in the trait that you can override, take a look at
// https://github.com/rust-mcp-stack/rust-mcp-sdk/blob/main/crates/rust-mcp-sdk/src/mcp_handlers/mcp_server_handler.rs

#[async_trait]
#[allow(unused)]
impl ServerHandler for DunnetHandler {
    // Handle ListToolsRequest, return list of available tools as ListToolsResult
    async fn handle_list_tools_request(
        &self,
        request: ListToolsRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: DunnetTools::tools(),
        })
    }

    /// Handles incoming CallToolRequest and processes it using the appropriate tool.
    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        // Attempt to convert request parameters into DunnetTools enum
        let tool_params: DunnetTools =
            DunnetTools::try_from(request.params).map_err(CallToolError::new)?;

        // Match the tool variant and execute its corresponding logic
        match tool_params {
            DunnetTools::DunnetWorldCommand(dunnet_world_command) => {
                dunnet_world_command.call_tool(&self.repl).await
            }
            DunnetTools::DunnetStartGameCommand(dunnet_start_game_command) => {
                dunnet_start_game_command.call_tool(&self.repl).await
            }
        }
    }
}
