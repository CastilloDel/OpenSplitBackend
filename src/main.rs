use actix_web::{post, web, App, HttpResponse, HttpServer};
use mongodb::{bson::doc, Client};
mod user;
use user::User;

/// Adds a new user to the "users" collection in the database.
#[post("/add_user")]
async fn add_user(client: web::Data<Client>) -> HttpResponse {
    let collection = client.database("OpenSplit").collection("Users");
    let user = User {
        first_name: String::from("a"),
        last_name: String::from("b "),
        username: String::from("c"),
        email: String::from("d"),
    };
    let result = collection.insert_one(user, None).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("MONGODB_URI").expect("You need to add the MONGODB_URI to the env");
    println!("{}", uri);

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    println!("Connected");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(add_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
