use actix_cors::Cors;
use actix_web::{get, post, put, web, App, HttpRequest, HttpResponse, HttpServer};
use auth::AuthorizationLevel;
use balance::{compute_balance_from_group, compute_user_balance_by_group};
use exchange::get_exchanges_from_group;
use futures::stream::StreamExt;
use mongodb::{bson::doc, options::IndexOptions, Client, IndexModel};
use schemas::{Expense, Group, UserNick};
use serde::{Deserialize, Serialize};
use std::env;

use crate::auth::check_authorization_level;
mod auth;
mod balance;
mod exchange;
mod schemas;

const DATABASE_NAME: &'static str = "OpenSplit";
const GROUP_COLLECTION_NAME: &'static str = "Groups";

fn check_if_user_is_in_group(user: &UserNick, group: &Group) -> bool {
    group
        .expenses
        .iter()
        .any(|expense| &expense.payer == user || expense.receivers.contains(&user))
}

#[derive(Deserialize, Serialize)]
struct GroupNameJson {
    name: String,
}

#[put("/groups/{id}")]
async fn add_group(
    request: HttpRequest,
    client: web::Data<Client>,
    id: web::Path<String>,
    json: web::Json<GroupNameJson>,
) -> HttpResponse {
    match check_authorization_level(request) {
        None => return HttpResponse::BadRequest().body("Authorization header was malformed"),
        Some(AuthorizationLevel::Frontend(_)) => {
            // We don't allow puts from the frontend
            return HttpResponse::Unauthorized().body("Missing permissions to carry on the query");
        }
        _ => {}
    };
    let groups = client
        .database(DATABASE_NAME)
        .collection(GROUP_COLLECTION_NAME);
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
async fn get_balance(
    request: HttpRequest,
    client: web::Data<Client>,
    id: web::Path<String>,
) -> HttpResponse {
    let authorization_level = check_authorization_level(request);
    if let None = authorization_level {
        return HttpResponse::BadRequest().body("Authorization header was malformed");
    };
    let groups = client
        .database(DATABASE_NAME)
        .collection(GROUP_COLLECTION_NAME);
    let group = match groups.find_one(doc! { "id": id.into_inner()}, None).await {
        Ok(Some(group)) => group,
        Ok(None) => return HttpResponse::NotFound().body("Couldn't find the desired group"),
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    if let Some(AuthorizationLevel::Frontend(nick)) = authorization_level {
        if check_if_user_is_in_group(&nick, &group) {
            return HttpResponse::Unauthorized().body("Missing permissions to carry on the query");
        }
    };
    HttpResponse::Ok().json(compute_balance_from_group(&group))
}

#[post("/groups/{id}/expenses")]
async fn add_expense(
    request: HttpRequest,
    client: web::Data<Client>,
    id: web::Path<String>,
    expense: web::Json<Expense>,
) -> HttpResponse {
    match check_authorization_level(request) {
        None => return HttpResponse::BadRequest().body("Authorization header was malformed"),
        Some(AuthorizationLevel::Frontend(_)) => {
            // We don't allow posts from the frontend
            return HttpResponse::Unauthorized().body("Missing permissions to carry on the query");
        }
        _ => {}
    };
    let groups = client
        .database(DATABASE_NAME)
        .collection::<Group>(GROUP_COLLECTION_NAME);
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

#[get("/users/{nick}/balance")]
async fn get_user_balance(
    request: HttpRequest,
    client: web::Data<Client>,
    id: web::Path<String>,
) -> HttpResponse {
    let id = id.into_inner();
    match check_authorization_level(request) {
        None => return HttpResponse::BadRequest().body("Authorization header was malformed"),
        Some(AuthorizationLevel::Frontend(nick)) if nick != id => {
            return HttpResponse::Unauthorized().body("Missing permissions to carry on the query")
        }
        _ => {}
    };
    let groups = client
        .database(DATABASE_NAME)
        .collection::<Group>(GROUP_COLLECTION_NAME);
    let groups_user_is_in: Vec<Group> = match groups
        .find(
            doc! { "expenses": { "$elemMatch": { "$or": [{"receivers": {"$in": [&id] }}, { "payer": &id}]}}},
            None,
        )
        .await
    {
        Ok(balance) => balance.collect::<Vec<_>>().await.into_iter().filter_map(|result| result.ok()).collect(),
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    HttpResponse::Ok().json(compute_user_balance_by_group(id, groups_user_is_in))
}

#[get("/groups/{id}/exchanges")]
async fn get_exchanges(
    request: HttpRequest,
    client: web::Data<Client>,
    id: web::Path<String>,
) -> HttpResponse {
    let authorization_level = check_authorization_level(request);
    if let None = authorization_level {
        return HttpResponse::BadRequest().body("Authorization header was malformed");
    };
    let groups = client
        .database(DATABASE_NAME)
        .collection::<Group>(GROUP_COLLECTION_NAME);
    let group = match groups.find_one(doc! { "id": id.into_inner()}, None).await {
        Ok(Some(group)) => group,
        Ok(None) => return HttpResponse::NotFound().body("Couldn't find the desired group"),
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    if let Some(AuthorizationLevel::Frontend(nick)) = authorization_level {
        if check_if_user_is_in_group(&nick, &group) {
            return HttpResponse::Unauthorized().body("Missing permissions to carry on the query");
        }
    };
    HttpResponse::Ok().json(get_exchanges_from_group(&group))
}

#[get("/groups/{id}/expenses")]
async fn get_expenses(
    request: HttpRequest,
    client: web::Data<Client>,
    id: web::Path<String>,
) -> HttpResponse {
    let authorization_level = check_authorization_level(request);
    if let None = authorization_level {
        return HttpResponse::BadRequest().body("Authorization header was malformed");
    };
    let groups = client
        .database(DATABASE_NAME)
        .collection::<Group>(GROUP_COLLECTION_NAME);
    let group = match groups.find_one(doc! { "id": id.into_inner()}, None).await {
        Ok(Some(group)) => group,
        Ok(None) => return HttpResponse::NotFound().body("Couldn't find the desired group"),
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    if let Some(AuthorizationLevel::Frontend(nick)) = authorization_level {
        if check_if_user_is_in_group(&nick, &group) {
            return HttpResponse::Unauthorized().body("Missing permissions to carry on the query");
        }
    };
    HttpResponse::Ok().json(&group.expenses)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = env::var("MONGODB_URI").expect("You need to add MONGODB_URI to the env");
    let _bot_token = env::var("BOT_API_TOKEN").expect("You need to add API_BOT_TOKEN to the env");

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    println!("Connected");
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "id": 1 })
        .options(options)
        .build();
    client
        .database(DATABASE_NAME)
        .collection::<Group>(GROUP_COLLECTION_NAME)
        .create_index(model, None)
        .await
        .expect("Database in an incosistent state");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(Cors::permissive())
            .service(add_group)
            .service(get_balance)
            .service(add_expense)
            .service(get_user_balance)
            .service(get_expenses)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
