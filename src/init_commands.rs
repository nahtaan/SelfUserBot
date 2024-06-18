#![allow(dead_code)]
pub mod init_commands {
    use std::fs;
    use std::fs::File;
    use std::ops::Deref;
    use std::sync::{RwLock};
    use config::Config;
    use lazy_static::lazy_static;
    use reqwest::StatusCode;
    use serde::Serialize;
    use serde_json::{json, Value, from_str};

    lazy_static! {
        static ref APPLICATION_ID: RwLock<String> = RwLock::new(String::new());
    }
    lazy_static! {
        static ref COMMAND_RESPONSES: RwLock<Vec<CommandResponse>> = RwLock::new(vec![]);
    }

    pub fn get_command_responses() -> Vec<CommandResponse> {
        let r = COMMAND_RESPONSES.read().unwrap();
        return r.clone();
    }

    pub fn get_application_id() -> String {
        let r = APPLICATION_ID.read().unwrap();
        return r.clone();
    }

    #[repr(u8)]
    pub enum IntegrationType {
        Guild,
        User
    }

    #[repr(u8)]
    pub enum InteractionContext {
        Guild,
        BotDm,
        PrivateChannel
    }
    impl IntegrationType {
        pub fn raw(&self) -> u8 {
            match &self {
                IntegrationType::Guild => {0}
                IntegrationType::User => {1}
            }
        }
    }
    impl InteractionContext {
        pub fn raw(&self) -> u8 {
            match &self {
                InteractionContext::Guild => {0}
                InteractionContext::BotDm => {1}
                InteractionContext::PrivateChannel => {2}
            }
        }
    }


    pub fn load_cmds(token: &str){
        let client = reqwest::blocking::Client::new();
        let auth_header = "Bot ".to_owned() + token;
        // send a request to discord to get the application id
        let application_response = client.get("https://discord.com/api/applications/@me")
            .header("Authorization", &auth_header)
            .send();

        // parse the body
        match application_response {
            Ok(resp) => {
                let text: String = resp.text().unwrap();
                let json: Value = from_str(text.as_str()).unwrap();
                let mut w = APPLICATION_ID.write().unwrap();
                *w = json.get("id").unwrap().as_str().unwrap().to_string();
                let _ = w.deref();
            }
            Err(_) => {}
        }
        let r = APPLICATION_ID.read().unwrap();
        println!("APPLICATION_ID: {}", r.as_str());
        // get the configurable commands from the config file and stores them in memory
        get_commands_from_file();

        // create a vec of CommandData for all the configured commands
        let mut command_data_to_send: Vec<CommandData> = Vec::new();
        let r = COMMAND_RESPONSES.read().unwrap();
        for response in r.to_vec() {
            command_data_to_send.push(
                CommandData {
                    name: response.name,
                    description: response.description,
                    integration_types: vec![IntegrationType::User],
                    contexts: vec![InteractionContext::BotDm, InteractionContext::Guild, InteractionContext::PrivateChannel],
                }
            );
        }
        let _ = r.deref();
        let mut body: String = "[".to_string();
        for data in command_data_to_send {
            body.push_str(data.to_body().as_str());
            body.push(',')
        }
        let _ = body.pop();
        body.push(']');
        let response = client.put("https://discord.com/api/applications/".to_owned() + get_application_id().as_str() + "/commands")
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .body(body)
            .send();
        match response {
            Ok(resp) => {
                if resp.status() == StatusCode::OK {
                    println!("Sent all commands to discord!");
                }else {
                    println!("Discord returned an error upon sending your commands.");
                    println!("{:#?}", resp.text());
                }
            }
            Err(err) => {
                println!("An error occurred whilst sending commands to discord! Error: {:?}", err);
            }
        }
    }

    fn get_commands_from_file() {
        // create the Commands.yml file if it doesn't already exist
        if !fs::metadata("Commands.yml").is_ok() {
            match File::create("Commands.yml") {
                Ok(_) => {}
                Err(_) => {
                    println!("Unable to create new Commands.yml file. Please check file permissions");
                    return;
                }
            }
        }

        // load the config file from disk
        let commands: Config = Config::builder()
            .add_source(config::File::with_name("Commands.yml"))
            .build()
            .expect("Failed to read Commands.yml. Please check file permissions.");
        let values = commands.cache.into_table().expect("Error parsing Commands.yml");

        let mut new_commands: Vec<CommandResponse> = Vec::new();
        // iterate over each command
        for (name, value) in values {
            // initialize defaults
            let mut description: String = "".to_string();
            let mut content: String = "".to_string();
            let mut embeds: Vec<MessageEmbed> = Vec::new();
            let mut components: Vec<ActionRow> = Vec::new();

            // collect values
            for (key, value) in value.into_table().unwrap() {
                // handle finding the description
                if key == "description"{
                    description = value.into_string().unwrap_or("".to_string());
                    // handle finding the content
                } else if key == "content" {
                    // collect value
                    content = value.into_string().unwrap_or("".to_string());
                    // handle finding the embeds
                } else if key == "embeds" {
                    // iterate over each embed
                    for (_, value) in value.into_table().unwrap() {
                        let mut title: Option<String> = None;
                        let mut description: Option<String> = None;
                        let mut url: Option<String> = None;
                        let mut color: Option<u32> = None;
                        let mut footer: Option<EmbedFooter> = None;
                        let mut image: Option<EmbedImage> = None;
                        let mut thumbnail: Option<EmbedThumbnail> = None;
                        let mut video: Option<EmbedVideo> = None;
                        let mut author: Option<EmbedAuthor> = None;
                        let mut fields: Option<Vec<EmbedField>> = None;

                        // collect values
                        for (id, value) in value.into_table().unwrap() {
                            match id.as_str() {
                                "title" => { title = Some(value.into_string().unwrap()) }
                                "description" => { description = Some(value.into_string().unwrap()) }
                                "url" => { url = Some(value.into_string().unwrap()) }
                                "color" => { color = Some(u32::from_str_radix(value.into_string().unwrap().as_str(), 16).unwrap()) }
                                "footer" => {
                                    for (id, value) in value.into_table().unwrap() {
                                        let mut text: String = "".to_string();
                                        let mut icon_url: Option<String> = None;
                                        match id.as_str() {
                                            "text" => { text = value.into_string().unwrap() }
                                            "icon_url" => { icon_url = Some(value.into_string().unwrap()) }
                                            &_ => {}
                                        }
                                        footer = Some(EmbedFooter { text, icon_url })
                                    }
                                }
                                "image" => {
                                    for (id, value) in value.into_table().unwrap() {
                                        if id == "url" {
                                            image = Some(EmbedImage { url: value.into_string().unwrap() })
                                        }
                                    }
                                }
                                "thumbnail" => {
                                    for (id, value) in value.into_table().unwrap() {
                                        if id == "url" {
                                            thumbnail = Some(EmbedThumbnail { url: value.into_string().unwrap() })
                                        }
                                    }
                                }
                                "video" => {
                                    for (id, value) in value.into_table().unwrap() {
                                        if id == "url" {
                                            video = Some(EmbedVideo { url: value.into_string().unwrap() })
                                        }
                                    }
                                }
                                "author" => {
                                    let mut name: String = String::from("");
                                    let mut url: Option<String> = None;
                                    let mut icon_url: Option<String> = None;
                                    for (id, value) in value.into_table().unwrap() {
                                        match id.as_str() {
                                            "name" => { name = value.into_string().unwrap() }
                                            "url" => { url = Some(value.into_string().unwrap()) }
                                            "icon_url" => { icon_url = Some(value.into_string().unwrap()) }
                                            &_ => {}
                                        }
                                    }
                                    author = Some(EmbedAuthor { name, url, icon_url });
                                }
                                "fields" => {
                                    // iterate over each field
                                    let mut new_fields: Vec<EmbedField> = vec![];
                                    for (_, value) in value.into_table().unwrap() {
                                        let mut name: String = String::from("");
                                        let mut valuee: String = String::from("");
                                        let mut inline = false;
                                        for (id, value) in value.into_table().unwrap() {
                                            match id.as_str() {
                                                "name" => { name = value.into_string().unwrap() }
                                                "value" => { valuee = value.into_string().unwrap() }
                                                "inline" => { inline = value.into_bool().unwrap() }
                                                &_ => {}
                                            }
                                        }
                                        new_fields.push(EmbedField { name, value: valuee, inline })
                                    }
                                    fields = Some(new_fields);
                                }
                                &_ => {}
                            }
                        }
                        embeds.push(MessageEmbed {
                            title: title.clone(),
                            description: description.clone(),
                            url: url.clone(),
                            color: color.clone(),
                            footer: footer.clone(),
                            image: image.clone(),
                            thumbnail: thumbnail.clone(),
                            video: video.clone(),
                            author: author.clone(),
                            fields: fields.clone(),
                        })
                    }
                } else if key == "buttons" {
                    let mut action_row_components: Vec<UrlButtonComponent> = vec![];
                    // iterate over each button
                    for (_, value) in value.into_table().unwrap() {
                        let mut label: String = String::from("");
                        let mut url: String = String::from("");
                        for (id, value) in value.into_table().unwrap() {
                            if id == "label" {
                                label = value.into_string().unwrap();
                            } else if id == "url" {
                                url = value.into_string().unwrap();
                            }
                        }
                        action_row_components.push(UrlButtonComponent {
                            r#type: 2,
                            style: 5,
                            label,
                            url,
                        })
                    }
                    components.push(ActionRow { r#type: 1, components: action_row_components });
                }
            }

            let command_data = CommandResponse {
                name,
                description,
                message: MessageData {
                    content,
                    embeds,
                    components,
                },
            };
            new_commands.push(command_data);
            // COMMAND_RESPONSES.lock().unwrap().push(command_data);
        }
        let mut w = COMMAND_RESPONSES.write().unwrap();
        *w = new_commands;
        let _ = w.deref();
    }

    pub struct CommandData {
        pub name: String,
        pub description: String,
        pub integration_types: Vec<IntegrationType>,
        pub contexts: Vec<InteractionContext>
    }

    impl CommandData {
        pub fn to_body(&self) -> String {
            let integration_types: Vec<u8> = self.integration_types
                    .iter()
                    .map(|integration| integration.raw())
                    .collect();
            let contexts: Vec<u8> =self.contexts
                    .iter()
                    .map(|context| context.raw())
                    .collect();
            let value: Value = json!({
                "name": self.name,
                "description": self.description,
                "integration_types": integration_types,
                "contexts": contexts
            });
            value.to_string()
        }
    }

    #[derive(Debug, Clone)]
    pub struct CommandResponse {
        pub name: String,
        pub description: String,
        pub message: MessageData
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct MessageData {
        pub content: String,
        pub embeds: Vec<MessageEmbed>,
        pub components: Vec<ActionRow>
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct MessageEmbed {
        pub title: Option<String>,
        pub description: Option<String>,
        pub url: Option<String>,
        pub color: Option<u32>,
        pub footer: Option<EmbedFooter>,
        pub image: Option<EmbedImage>,
        pub thumbnail: Option<EmbedThumbnail>,
        pub video: Option<EmbedVideo>,
        pub author: Option<EmbedAuthor>,
        pub fields: Option<Vec<EmbedField>>
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct EmbedFooter {
        pub text: String,
        pub icon_url: Option<String>
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct EmbedImage {
        pub url: String
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct EmbedThumbnail {
        pub url: String
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct EmbedVideo {
        pub url: String
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct EmbedAuthor {
        pub name: String,
        pub url: Option<String>,
        pub icon_url: Option<String>
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct EmbedField {
        pub name: String,
        pub value: String,
        pub inline: bool
    }

    // This will only support url buttons for now as there is no good (easy) way of configuring functionality for custom_id buttons
    #[derive(Debug, Clone, Serialize)]
    pub struct ActionRow {
        pub r#type: u8,
        pub components: Vec<UrlButtonComponent>
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct UrlButtonComponent {
        pub r#type: u8,
        pub style: u8,
        pub label: String,
        pub url: String
    }
}