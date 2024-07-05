use anyhow::Context;
use fastembed::{InitOptions, TextEmbedding};
use qdrant_client::qdrant::SearchPoints;
use qdrant_client::Qdrant;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;

#[macro_use] extern crate rocket;

#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("./static/index.html").await
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SearchRequest {
    input: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SearchResponse {
    results: Vec<String>,
}

#[post("/search", format="json", data="<request>")]
async fn search_qdrant(request: Json<SearchRequest>) -> Result<Json<SearchResponse>, std::io::Error>  {
    let input = request.input.clone();

    // benchmark -> create new model for each request or use a global model
    let model = TextEmbedding::try_new(InitOptions {
        model_name: fastembed::EmbeddingModel::BGELargeENV15,
        show_download_progress: true,
        // todo -> set diamension to 1024
        ..Default::default()
    })
    .context("Failed to load the sentence transform model")
    .unwrap();
    let query_embedding = model
        .embed(vec![input.clone()], None)
        .context("Failed to embedd query")
        .unwrap();

    // benchmark -> create new client for each request or use a global client
    let qdrant_client = Qdrant::from_url("http://localhost:6334")
        .build()
        .context("Failed to create qdrant client")
        .unwrap();
    let collection_name = "law_search";
    let search_point = SearchPoints {
        collection_name: collection_name.to_string(),
        vector: query_embedding[0].clone(),
        limit: 3,
        with_payload: Some(true.into()),
        ..Default::default()
    };
    let search_result: qdrant_client::qdrant::SearchResponse = qdrant_client
        .search_points(search_point)
        .await
        .context("Failed to search qdrant")
        .unwrap();
    let results = search_result.result
        .iter()
        .map(|r| r.payload.get("text").unwrap().to_string())
        .collect();
    Ok(SearchResponse {
        results,
    }.into())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, search_qdrant])
}
