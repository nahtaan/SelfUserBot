use std::{fs, thread};
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use config::Config;
use serde_json::Value;
use crate::handle_responses::handle_responses::handle_responses;
use crate::interaction_endpoint::command_handler::Interaction;

mod init_commands;
mod interaction_endpoint;
mod handle_responses;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !fs::metadata("Config.yml").is_ok() {
        match File::create("Config.yml") {
            Ok(_) => {}
            Err(_) => {
                println!("Unable to create new config file. Please check file permissions");
                return Ok(());
            }
        }
    }
    let settings = Config::builder()
        .add_source(config::File::with_name("Config.yml"))
        .build()
        .unwrap();

    // get webserver options
    let webserver_section: Value = settings.get::<Value>("webserver").unwrap();

    // get the address
    let binding = webserver_section.get("address").unwrap().to_owned();
    let address = binding.as_str().unwrap();

    // get the port
    let binding = webserver_section.get("port").unwrap().to_owned();
    let port: u16 = u16::try_from(binding.as_u64().unwrap()).unwrap();

    // get the publickey
    let binding = settings.get::<Value>("discord").unwrap().to_owned().get("publickey").unwrap().to_owned();
    let publickey= binding.as_str().unwrap();
    // get the bot token
    let binding = settings.get::<Value>("discord").unwrap().to_owned().get("token").unwrap().to_owned();
    let token = binding.as_str().unwrap();

    // send commands to discord
    init_commands::init_commands::load_cmds(&token);

    // setup threads for handling interactions

    // create a channel to allow the endpoint to tell the handler about interactions
    let (tx, rx): (Sender<Interaction>, Receiver<Interaction>) = mpsc::channel();

    // start the handler thread
    thread::spawn(|| {
        handle_responses(rx, 5);
    });


    // start the webserver
    unsafe {
        interaction_endpoint::command_handler::main(address, port, publickey, tx)
            .expect("There was an error that occurred when running the interactions endpoint.");
    }

    Ok(())
}