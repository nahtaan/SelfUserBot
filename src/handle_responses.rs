pub mod handle_responses {
    use std::sync::mpsc::{Receiver};
    use reqwest::StatusCode;
    use threadpool::ThreadPool;
    use crate::init_commands::init_commands::{get_application_id, get_command_responses};
    use crate::interaction_endpoint::command_handler::Interaction;

    pub fn handle_responses(receiver: Receiver<Interaction>, threads: u8){
        let pool = ThreadPool::new(threads as usize);
        loop {
            match receiver.recv() {
                Ok(interaction) => {
                    pool.execute(move || {
                        let data = &interaction.data.unwrap();
                        let name = &data.name;
                        for response in get_command_responses() {
                            if response.name.as_str().eq(name.as_str()) {
                                let client = reqwest::blocking::Client::new();
                                let body = serde_json::to_string(&response.message).unwrap();
                                let url = "https://discord.com/api/webhooks/".to_owned() + get_application_id().as_str() + "/" + &interaction.token + "/messages/@original";
                                let response = client.patch(url)
                                    .header("Content-Type", "application/json")
                                    .body(body)
                                    .send();
                                match response {
                                    Ok(resp) => {
                                        let status = &resp.status();
                                        if status != &StatusCode::OK {
                                            println!("There was an error whilst responding to a command! {}", status);
                                        }
                                    }
                                    Err(err) => {
                                        println!("Failed to respond to command: {:?}", err);
                                    }
                                }
                                break;
                            }
                        }
                    });
                }
                Err(err) => {
                    println!("{:#?}", err);
                    break
                }
            }
        }
    }

}