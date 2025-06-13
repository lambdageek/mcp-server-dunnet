use rust_mcp_sdk::schema::{self, CallToolResult, schema_utils::CallToolError};
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
pub struct DunnetWorldCommand {
    /// The command to send to the dunnet world.
    command: String,
}

/// Dunnet start game command
/// Sends a command to the dunnet world and returns the result.
#[mcp_tool(
    name = "dunnet_start_game",
    description = "Starts a new game of Dunnet. Run this command exactly once before running any other dunnet_world_command.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = true,
    read_only_hint = false
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct DunnetStartGameCommand {}

impl DunnetWorldCommand {
    pub async fn call_tool(&self, repl: &DunnetRepl) -> Result<CallToolResult, CallToolError> {
        let response = repl.interact(self.command.clone())
            .await;
        let response_message = match response {
            crate::DunnetResponse::Done(lines) => lines.join("\n") + "\nThe game has ended. You can not send any more commands.",
            crate::DunnetResponse::Output(lines) => lines.join("\n") + "\nWhat is your next command?",
        };
        let text_response = schema::TextContent::new(response_message, Some(schema::Annotations {
            audience: vec![schema::Role::User, schema::Role::Assistant],
            priority: None,
        }));
        Ok(CallToolResult {
            content: vec![text_response.into()],
            is_error: None,
            meta: None,
        })
    }
}

impl DunnetStartGameCommand {
    pub async fn call_tool(&self, repl: &DunnetRepl) -> Result<CallToolResult, CallToolError> {
        let response = repl.game_start().await;
        let response_message = match response {
            crate::DunnetResponse::Done(lines) => lines.join("\n") + "\nThe game has ended. You can not send any more commands.",
            crate::DunnetResponse::Output(lines) => lines.join("\n") + "\nWhat is your next command?",
        };
        let text_response = schema::TextContent::new(response_message, Some(schema::Annotations {
            audience: vec![schema::Role::User, schema::Role::Assistant],
            priority: None,
        }));
        Ok(CallToolResult {
            content: vec![text_response.into()],
            is_error: None,
            meta: None,
        })
    }
}

// Generates an enum names DunnetTools, with DunnetWorldTool variant
tool_box!(DunnetTools, [DunnetWorldCommand, DunnetStartGameCommand]);
