use alith::{Agent, EmbeddingsBuilder, InMemoryStorage, LLM};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let model = LLM::from_model_name("gpt-4")?;
    let embeddings_model = model.embeddings_model("text-embedding-ada-002");
    let data = EmbeddingsBuilder::new(embeddings_model.clone())
        .documents(vec!["doc0", "doc1", "doc2"])
        .unwrap()
        .build()
        .await?;
    let storage = InMemoryStorage::from_multiple_documents(embeddings_model, data);

    let agent = Agent::new("simple agent", model, vec![])
        .preamble(
            r#"
You are a dictionary assistant here to assist the user in understanding the meaning of words.
You will find additional non-standard word definitions that could be useful below.
"#,
        )
        .store_index(1, storage);
    let response = agent.prompt("What does \"glarb-glarb\" mean?").await?;

    println!("{}", response);

    Ok(())
}
