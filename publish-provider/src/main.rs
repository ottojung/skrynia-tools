use skrynia_publish_provider::{config, server};

#[tokio::main]
async fn main() {
    let cfg = match config::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("configuration error: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = server::run(cfg).await {
        eprintln!("provider error: {err}");
        std::process::exit(1);
    }
}
