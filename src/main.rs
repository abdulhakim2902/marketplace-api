#[tokio::main]
async fn main() {
    let (worker, http_server) = nft_aggregator_api::init()
        .await
        .expect("Failed to initialize server");

    let (worker_res, http_server_res) = tokio::join! {
        worker.start(),
        http_server.start(),
    };

    tracing::info!("Worker finished with: {:?}", worker_res);
    tracing::info!("HTTP server finished with: {:?}", http_server_res);
}
