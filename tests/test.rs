#[cfg(test)]
mod tests {
    use actix_web::{get, web, App, Error, HttpResponse, Responder};
    use std::fs::File;
    use std::io::Read;
    use unlimited_poker::bootstrap::{bootstrap, bootstrap_schema};
    use url::Url;
    use websocket::client::ClientBuilder;
    use websocket::{Message, OwnedMessage};
    use std::sync::mpsc::channel;
    use std::io::stdin;
    //use std::sync::mpsc::channel;
    use std::thread;

    const CONNECTION: &'static str = "ws://localhost:8097/ws";

    #[get("/")]
    async fn my_handler() -> Result<impl Responder, Error> {
        Ok(HttpResponse::Ok())
    }

    #[actix_web::test]
    async fn test_ws_get() {
        println!("Connecting to {}", CONNECTION);

        let client = ClientBuilder::new(CONNECTION)
            .unwrap()
            .add_protocol("graphql-ws")
            .connect_insecure()
            .unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();

        let (tx, rx) = channel();

        let tx_1 = tx.clone();

        	let send_loop = thread::spawn(move || {
		loop {
			// Send loop
			let message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					let _ = sender.send_message(&message);
					// If it's a close message, just send it and then return.
					return;
				}
				_ => (),
			}
			// Send the message
			match sender.send_message(&message) {
				Ok(()) => (),
				Err(e) => {
					println!("Send Loop: {:?}", e);
					let _ = sender.send_message(&Message::close());
					return;
				}
			}
		}
	});

	let receive_loop = thread::spawn(move || {
		// Receive loop
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(e) => {
					println!("Receive Loop: {:?}", e);
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					// Got a close message, so send a close message and return
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
				OwnedMessage::Ping(data) => {
					match tx_1.send(OwnedMessage::Pong(data)) {
						// Send a pong in response
						Ok(()) => (),
						Err(e) => {
							println!("Receive Loop: {:?}", e);
							return;
						}
					}
				}
				// Say what we received
				_ => println!("Receive Loop: {:?}", message),
			}
		}
	});

        println!("Successfully connected");
    }

    #[actix_web::test]
    async fn test_index_get() {
        let schema = bootstrap_schema().await.unwrap();
        let mut srv = actix_test::start(move || {
            App::new()
                .app_data(web::Data::new(schema.clone()))
                .configure(bootstrap)
        });
        //let res = srv.post("/ws").send().await.unwrap();
        //println!("res: {:?}", res);
        let mut file = File::open("tests/test.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let json: serde_json::Value =
            serde_json::from_str(&data).expect("JSON was not well-formatted");

        let res = srv
            .post("/graphql")
            .append_header(("x-user-token", "sean"))
            .append_header(("x-table-token", 123))
            .send_json(&json)
            .await
            .unwrap();

        println!("res: {:?}", res);
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_ws() {
        println!("Connecting to {}", CONNECTION);

        let client = ClientBuilder::new(CONNECTION)
            .unwrap().add_protocol("graphql-ws").connect_insecure().unwrap();
            //.connect_insecure()
            //.unwrap();

    }
}
