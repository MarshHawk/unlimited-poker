#[cfg(test)]
mod tests {
    use actix_web::{get, web, App, Error, HttpResponse, Responder};
    use unlimited_poker::bootstrap::{bootstrap, bootstrap_schema};
    use std::io::Read;
    use std::fs::File;

    #[get("/")]
    async fn my_handler() -> Result<impl Responder, Error> {
        Ok(HttpResponse::Ok())
    }

    #[actix_web::test]
    async fn test_index_get() {
        let schema = bootstrap_schema().await.unwrap();
        let mut srv = actix_test::start(move || {
            App::new()
                .app_data(web::Data::new(schema.clone()))
                .configure(bootstrap)
        });

        let mut ws = srv.ws().await.unwrap();

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

}
