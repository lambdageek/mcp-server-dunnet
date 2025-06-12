//! Model Context Protocol server stuff for Dunnet

mod handler {
    use async_trait::async_trait;
    use rust_mcp_sdk::schema::{
        schema_utils::CallToolError, CallToolRequest, CallToolResult, ListToolsRequest,
        ListToolsResult, RpcError,
    };
    use rust_mcp_sdk::{mcp_server::ServerHandler, McpServer};

    use super::tools::GreetingTools;

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
                tools: GreetingTools::tools(),
            })
        }

        /// Handles incoming CallToolRequest and processes it using the appropriate tool.
        async fn handle_call_tool_request(
            &self,
            request: CallToolRequest,
            runtime: &dyn McpServer,
        ) -> std::result::Result<CallToolResult, CallToolError> {
            // Attempt to convert request parameters into GreetingTools enum
            let tool_params: GreetingTools =
                GreetingTools::try_from(request.params).map_err(CallToolError::new)?;

            // Match the tool variant and execute its corresponding logic
            match tool_params {
                GreetingTools::SayHelloTool(say_hello_tool) => say_hello_tool.call_tool(),
                GreetingTools::SayGoodbyeTool(say_goodbye_tool) => say_goodbye_tool.call_tool(),
            }
        }
    }
}

mod tools {
    use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult};
    use rust_mcp_sdk::{
        macros::{mcp_tool, JsonSchema},
        tool_box,
    };

    //****************//
    //  SayHelloTool  //
    //****************//
    #[mcp_tool(
        name = "say_hello",
        description = "Accepts a person's name and says a personalized \"Hello\" to that person",
        idempotent_hint = false,
        destructive_hint = false,
        open_world_hint = false,
        read_only_hint = false
    )]
    #[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
    pub struct SayHelloTool {
        /// The name of the person to greet with a "Hello".
        name: String,
    }

    impl SayHelloTool {
        pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
            let hello_message = format!("Hello, {}!", self.name);
            Ok(CallToolResult::text_content(hello_message, None))
        }
    }

    //******************//
    //  SayGoodbyeTool  //
    //******************//
    #[mcp_tool(
        name = "say_goodbye",
        description = "Accepts a person's name and says a personalized \"Goodbye\" to that person.",
        idempotent_hint = false,
        destructive_hint = false,
        open_world_hint = false,
        read_only_hint = false
    )]
    #[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
    pub struct SayGoodbyeTool {
        /// The name of the person to say goodbye to.
        name: String,
    }
    impl SayGoodbyeTool {
        pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
            let hello_message = format!("Goodbye, {}!", self.name);
            Ok(CallToolResult::text_content(hello_message, None))
        }
    }

    //******************//
    //  GreetingTools  //
    //******************//
    // Generates an enum names GreetingTools, with SayHelloTool and SayGoodbyeTool variants
    tool_box!(GreetingTools, [SayHelloTool, SayGoodbyeTool]);

}

use handler::MyServerHandler;
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
    LATEST_PROTOCOL_VERSION,
};

use rust_mcp_sdk::{
    error::SdkResult,
    mcp_server::{server_runtime, ServerRuntime},
    McpServer, StdioTransport, TransportOptions,
};

pub async fn server_main() -> SdkResult<()> {
    // STEP 1: Define server details and capabilities
    let server_details = InitializeResult {
        // server name and version
        server_info: Implementation {
            name: "Hello World MCP Server".to_string(),
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