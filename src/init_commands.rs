pub mod init_commands {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    pub async fn load_cmds(token: &str){
        let client = reqwest::Client::new();
        let json_response = client.get("https://discord.com/api/applications/@me")
            .header("Authorization", "Bot ".to_owned() + token)
            .send()
            .await
            .expect("Something went wrong!")
            .text()
            .await.expect("Something went wrong!");
        let v: Value = serde_json::from_str(json_response.as_str()).expect("Error parsing json");
        let application_id: &str = v["id"].as_str().unwrap();
        let body: String = Command {
            name: String::from("testcmd"),
            description: String::from("A command for testing purposes!"),
            is_user_installable: true
        }.to_body().await.expect("Could not parse command.");
        let cmd_response = client.post("https://discord.com/api/applications/".to_owned()+application_id+"/commands")
            .header("Authorization", "Bot ".to_owned() + token)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Oh no!")
            .text()
            .await.expect("Oh no!");
        println!("Command Created: {}", cmd_response)
    }

    #[derive(Debug)]
    pub struct Command {
        name: String,
        description: String,
        is_user_installable: bool
    }

    impl Command {
        async fn to_body(&self) -> Result<String, serde_json::Error>{
            let mut vec = Vec::new();
            if self.is_user_installable {
                vec.push(1);
            }else{
                vec.push(0);
            }
            let cmd_data = CommandData {
                name: self.name.clone(),
                description: self.description.clone(),
                integration_types: vec,
                contexts: vec![0,1,2]
            };
            let raw = serde_json::to_string(&cmd_data);
            return raw;
        }
    }

    #[derive(Serialize, Deserialize)]
    struct CommandData {
        name: String,
        description: String,
        integration_types: Vec<u8>,
        contexts: Vec<u8>
    }
}