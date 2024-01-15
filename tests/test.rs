use actix_web::{
    get, guard, http::header::HeaderMap, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder, Result,
};
//use actix_gql_ws_stream::*;

#[cfg(test)]
mod tests {
    //use super::*;

    use std::{fs::File, collections::HashMap};
    use std::io::Read;

    use actix_web::{test, web, App, HttpServer};
    use rustc_serialize::json::Json;
    use unlimited_poker::bootstrap::{bootstrap, bootstrap_schema};

    #[actix_web::test]
    async fn test_index_get() {
        let schema = bootstrap_schema().await.unwrap();

        //let schema = HttpServer::new(move || App::new().app_data(web::Data::new(schema.clone())).configure(bootstrap));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(schema.clone()))
                .configure(bootstrap),
        )
        .await;

        let mut file = File::open("tests/test.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let json: serde_json::Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
        
        let req = test::TestRequest::post()
            .uri("/graphql")
            .append_header(("x-user-token", "sean"))
            .append_header(("x-table-token", 123))
            .set_json(json)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("resp: {:?}", resp);
        assert!(resp.status().is_success());
    }
}
