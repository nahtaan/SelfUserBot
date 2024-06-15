mod init_commands;
mod handle_commands;

// use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // send commands to discord
    init_commands::init_commands::load_cmds("put yer token 'ere")
        .await;

    // start the webserver
    handle_commands::command_handler::setup();

    Ok(())
}