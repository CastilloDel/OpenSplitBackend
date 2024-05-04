use actix_web::{get, post, put, web, App, HttpResponse, HttpServer};
use mongodb::{bson::doc, Client};
mod schemas;
use schemas::Group;
use serde::{Deserialize, Serialize};

use crate::{balance::compute_balance_from_group, schemas::Expense};
mod balance;

#[derive(Deserialize, Serialize)]
struct GroupNameJson {
    name: String,
}

#[put("/groups/{id}")]
async fn add_group(
    client: web::Data<Client>,
    id: web::Path<String>,
    json: web::Json<GroupNameJson>,
) -> HttpResponse {
    let groups = client.database("OpenSplit").collection("Groups");
    let group = Group {
        name: json.into_inner().name,
        id: id.into_inner(),
        expenses: vec![],
    };
    match groups.insert_one(group, None).await {
        Ok(_) => HttpResponse::Ok().body("Group added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/groups/{id}/balance")]
async fn get_balance(client: web::Data<Client>, id: web::Path<String>) -> HttpResponse {
    let groups = client.database("OpenSplit").collection("Groups");
    match groups.find_one(doc! { "id": id.into_inner()}, None).await {
        Ok(Some(group)) => HttpResponse::Ok().json(compute_balance_from_group(group)),
        Ok(None) => HttpResponse::NotFound().body("Couldn't find the desired group"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[post("/groups/{id}/expenses")]
async fn add_expense(
    client: web::Data<Client>,
    id: web::Path<String>,
    expense: web::Json<Expense>,
) -> HttpResponse {
    let groups = client.database("OpenSplit").collection::<Group>("Groups");
    let id = id.into_inner();
    match groups
        .update_one(
            doc! { "id": id},
            doc! { "$push": { "expenses": bson::to_bson(&expense.into_inner()).unwrap()}},
            None,
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().body("Expense added"),
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("MONGODB_URI").expect("You need to add the MONGODB_URI to the env");
    println!("Using the following URI: {}", uri);

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    println!("Connected");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(add_group)
            .service(get_balance)
            .service(add_expense)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
