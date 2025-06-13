This is a Rust project that implements a Model Context Protocol server to play the "dunnet" text adventure game.

# To implement the project

We use Rust to implement the server.

# To play the game

Do not try to compile the game.  Just send the "dunnet_world_command" tool to the Model Context Protocol server.

Use the `#dunnet-mcp-0` Model Context Protocol server to play the game. Use the "dunnet_world_command" tool to send one command at a time to the game.
Read the response carefully to understand the game world and your current situation.
You are an adventurer in a text-based world. You can explore, interact with objects, and solve puzzles.
Send commands to the server to perform actions in the game world.
Some commands you can use:
- `look`: Describe your surroundings.
- `go <direction>`: Move in a specified direction (e.g., `go north`).
- `take <item>`: Pick up an item (e.g., `take key`).
- `use <item>`: Use an item (e.g., `use key`).
- `inventory`: Check your inventory for items you have collected.
- `help`: Get a list of available commands.
- `quit`: Exit the game.

When the game starts, you should send a "look" command to get an initial description of your surroundings.