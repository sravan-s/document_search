use std::fs;

use anyhow::Context;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use ort::ROCmExecutionProvider;
use qdrant_client::{
    qdrant::{CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder},
    Payload, Qdrant,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let model = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::BGELargeENV15,
        show_download_progress: true,
        execution_providers: vec![
            ROCmExecutionProvider::default().with_device_id(0).build(),
        ],
        // todo -> set diamension to 1024
        ..Default::default()
    })
    .context("Failed to load the model")
    .unwrap();

    let qdrant_client = Qdrant::from_url("http://localhost:6334")
        .build()
        .context("Failed to create qdrant client")
        .unwrap();

    let collection_name = "law_search";
    let law_search_exist = qdrant_client
        .collection_exists(collection_name)
        .await
        .context("Failed to list collections")
        .unwrap();

    if !law_search_exist {
        qdrant_client
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine)),
            )
            .await
            .context("Failed to create collection")
            .unwrap();
    }

    let directory = fs::read_dir("../data/formatted").unwrap();
    for file in directory {
        let d = fs::read_to_string(file.unwrap().path()).unwrap();
        let (ids, docs): (Vec<String>, Vec<String>) = d
            .split("$$$$$")
            .map(|x| {
                let id = x.split("\n").into_iter().nth(1).unwrap_or("");
                (id.to_string(), x.to_string())
            })
            .filter(|(id, doc)| id.len() > 0 && doc.len() > 0)
            .unzip();

        let embeddings = model
            .embed(docs.clone(), None)
            .context("Failed to embedd docs")
            .unwrap();

        
        for (idx, id) in ids.iter().enumerate() {
            let mut points: Vec<PointStruct> = vec![];
            let embedding = embeddings.get(idx).unwrap();
            let text = docs.get(idx).unwrap();
            let payload: Payload = json!({
                "id": id.to_string(),
                "text": text.to_string(),
            }).try_into().unwrap();
            let point = PointStruct::new(
                Uuid::new_v4().to_string(),
                embedding.to_owned(),
                payload.clone(),
            );
            points.push(point);
            let operation_info = qdrant_client
                .upsert_points(UpsertPointsBuilder::new(collection_name, points))
                .await;
            println!("{:?}", operation_info);
        }
    }
}
