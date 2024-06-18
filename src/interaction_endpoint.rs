pub mod command_handler {
    use std::sync::mpsc::Sender;
    use std::time::Duration;
    use actix_web::{App, HttpResponse, HttpServer, post, Responder};
    use actix_web::http::header::HeaderMap;
    use actix_web::web::{Bytes};
    use ed25519_dalek::{PublicKey, Signature, SignatureError, Verifier};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct InteractionOption {
        pub name: String,
        pub r#type: u8,
        pub value: Option<String>
    }

    #[derive(Serialize, Deserialize)]
    pub struct InteractionData {
        pub id: String,
        pub name: String,
        pub r#type: u8,
        pub options: Option<Vec<InteractionOption>>
    }

    #[derive(Serialize, Deserialize)]
    pub struct Interaction {
        pub id: String,
        pub application_id: String,
        pub r#type: u8,
        pub token: String,
        pub data: Option<InteractionData>
    }


    #[post("/")]
    pub async fn post_interaction(req: actix_web::HttpRequest, bytes: Bytes) -> impl Responder {
        let body = String::from_utf8(bytes.to_vec()).map_err(|_| HttpResponse::BadRequest().finish()).unwrap();
        unsafe {
            match validate_discord_signature(req.headers(), &body, &(PUB_KEY.unwrap())) {
                Ok(_) => {
                    let interaction: Interaction = serde_json::from_str(&body.as_str()).unwrap();
                    if &interaction.r#type == &u8::from(1u8) {
                        HttpResponse::Ok()
                            .insert_header(("Content-Type", "application/json"))
                            .body("{\"type\": 1}")
                    } else {
                        let _ = &SENDER.clone().unwrap().send(interaction);
                        HttpResponse::Ok()
                            .insert_header(("Content-Type", "application/json"))
                            .body("{\"type\": 5}")
                    }
                }
                Err(_) => {
                    println!("Invalid discord signature");
                    HttpResponse::MethodNotAllowed()
                        .body("invalid request signature")
                }
            }
        }
    }
    static mut PUB_KEY: Option<PublicKey> = None;
    static mut SENDER: Option<Sender<Interaction>> = None;

    #[actix_web::main]
    pub async unsafe fn main(address: &str, port: u16, publickey: &str, sender: Sender<Interaction>) -> std::io::Result<()> {
        PUB_KEY = Some(PublicKey::from_bytes(
            &hex::decode(publickey)
                .expect("Invalid Discord publickey")
        ).expect("Failed to create Discord publickey"));
        SENDER = Some(sender);
        HttpServer::new(|| {
            App::new()
                .service(post_interaction)
        })
            .keep_alive(Duration::from_secs(75))
            .bind((address, port))?
            .run()
            .await
    }

    pub fn validate_discord_signature(headers: &HeaderMap, body: &String, pub_key: &PublicKey) -> Result<(), SignatureError> {
        let sig_ed25519 = {
            let header_signature = headers.get("X-Signature-Ed25519");
            if header_signature.is_none(){
                println!("x-signature-ed25519 was not found");
                return Err(SignatureError::new());
            }
            let decoded_header = match hex::decode(header_signature.unwrap()) {
                Ok(v) => {
                    v
                }
                Err(_) => {
                    return Err(SignatureError::new())
                }
            };

            let mut sig_arr: [u8; 64] = [0; 64];
            for (i, byte) in decoded_header.into_iter().enumerate() {
                sig_arr[i] = byte;
            }
            Signature::from_bytes(sig_arr.as_slice())
        };
        let sig_timestamp = headers.get("X-Signature-Timestamp");
        if sig_timestamp.is_none() {
            println!("x-signature-timestamp was not found");
            return Err(SignatureError::new());
        }
        let content = sig_timestamp.unwrap()
            .as_bytes()
            .iter()
            .chain(body.as_bytes().iter())
            .cloned()
            .collect::<Vec<u8>>();
        pub_key
            .verify(&content.as_slice(), &(sig_ed25519.unwrap()))
    }
}