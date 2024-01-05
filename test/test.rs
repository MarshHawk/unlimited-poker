use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, http::header::HeaderMap, Result, Responder, get};

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { count: 4 }))
                .service(index),
        )
        .await;
        //let req = test::TestRequest::get().uri("/").to_request();
        //let resp: AppState = test::call_and_read_body_json(&app, req).await;
//
        //assert_eq!(resp.count, 4);
    }
}