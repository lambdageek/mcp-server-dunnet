use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError};
use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    tool_box,
};

use crate::DunnetRepl;

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
    pub async fn call_tool(&self, repl: &DunnetRepl) -> Result<CallToolResult, CallToolError> {
        let response = repl.interact(self.command.clone())
            .await;
        let response_message = match response {
            crate::DunnetResponse::Done(lines) => lines.join("\n") + "\nThe game has ended. You can not send any more commands.",
            crate::DunnetResponse::Output(lines) => lines.join("\n") + "\nWhat is your next command?",
        };
        Ok(CallToolResult::text_content(response_message, None))
    }
}

// Generates an enum names DunnetTools, with DunnetWorldTool variant
tool_box!(DunnetTools, [DunnetWorldTool]);
