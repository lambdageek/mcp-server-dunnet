//! Model Context Protocol server stuff for Dunnet

mod handler {
    use async_trait::async_trait;
    use rust_mcp_sdk::schema::{
        CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, RpcError,
        schema_utils::CallToolError,
    };
    use rust_mcp_sdk::{McpServer, mcp_server::ServerHandler};

    use super::tools::DunnetTools;

    // Custom Handler to handle MCP Messages
    pub struct MyServerHandler;

    // To check out a list of all the methods in the trait that you can override, take a look at
    // https://github.com/rust-mcp-stack/rust-mcp-sdk/blob/main/crates/rust-mcp-sdk/src/mcp_handlers/mcp_server_handler.rs

    #[async_trait]
    #[allow(unused)]
    impl ServerHandler for MyServerHandler {
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
                DunnetTools::DunnetWorldTool(dunnet_world_tool) => {
                    dunnet_world_tool.call_tool().await
                }
            }
        }
    }
}

mod tools {
    use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError};
    use rust_mcp_sdk::{
        macros::{JsonSchema, mcp_tool},
        tool_box,
    };

    /// Dunnet world command
    /// Sends one command to the dunnet world
    #[mcp_tool(
        name = "dunnet_world_command",
        description = "Accepts a dunnet game command and sends it to the world.  Returns a description of the world after the command is executed. Read the result carefully, as it may contain important information about the game state. If the command changed the world you may need to use the 'look' command again to see the difference.",
        idempotent_hint = false,
        destructive_hint = false,
        open_world_hint = true,
        read_only_hint = false
    )]
    #[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
    pub struct DunnetWorldTool {
        /// The command to send to the dunnet world.
        command: String,
    }

    impl DunnetWorldTool {
        pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
            let response_message = format!("Sending command to dunnet world: {}", self.command);
            Ok(CallToolResult::text_content(response_message, None))
        }
    }

    // Generates an enum names DunnetTools, with DunnetWorldTool variant
    tool_box!(DunnetTools, [DunnetWorldTool]);
}

use handler::MyServerHandler;
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
    ServerCapabilitiesTools,
};

use rust_mcp_sdk::{
    McpServer, StdioTransport, TransportOptions,
    error::SdkResult,
    mcp_server::{ServerRuntime, server_runtime},
};

pub async fn server_main() -> SdkResult<()> {
    // STEP 1: Define server details and capabilities
    let server_details = InitializeResult {
        // server name and version
        server_info: Implementation {
            name: "Dunnet MCP Server".to_string(),
            version: "0.1.0".to_string(),
        },
        capabilities: ServerCapabilities {
            // indicates that server support mcp tools
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default() // Using default values for other fields
        },
        meta: None,
        instructions: Some("use this server to play dunnet.  send one command at a time. some commands you can use are: look, go <direction>, take <object>, use <object>.  use the command 'quit' to exit the game.".to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // STEP 2: create a std transport with default options
    let transport = StdioTransport::new(TransportOptions::default())?;

    // STEP 3: instantiate our custom handler for handling MCP messages
    let handler = MyServerHandler {};

    // STEP 4: create a MCP server
    let server: ServerRuntime = server_runtime::create_server(server_details, transport, handler);

    // STEP 5: Start the server
    server.start().await
}
