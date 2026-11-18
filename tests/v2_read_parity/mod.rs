mod cases;
mod fixture;
mod positive;
mod process;
#[test]
#[ignore = "requires pinned, separately built Skrynia v2 binary"]
fn pinned_v2_http_reads_match_tools_facade() {
    let bin = std::env::var("SKRYNIA_V2_SERVER_BIN").expect("SKRYNIA_V2_SERVER_BIN");
    let server = process::Server::start(&bin).unwrap();
    cases::verify(&server.url).unwrap();
    positive::verify(&server).unwrap();
}
