mod app;
mod auth;
mod config;
mod forward;
mod run;
mod state;
mod token;

#[tokio::main]
async fn main() {
    if let Err(err) = run::run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
