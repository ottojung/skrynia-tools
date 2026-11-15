pub fn print() {
    println!("skrynia app new --name NAME --directory DIR");
    println!(
        "skrynia app deploy --name NAME --repo GITHUB_URL --subdir PATH --apps-dir DIR [--commit SHA]"
    );
    println!("skrynia app releases|inspect --name NAME --apps-dir DIR");
    println!("skrynia app rollback --name NAME --release ID --apps-dir DIR");
    println!(
        "skrynia object get|create|replace|delete|list --key KEY --url URL [--token-file FILE]"
    );
    println!("skrynia log init|append|read --prefix PREFIX --url URL [--token-file FILE]");
    println!("skrynia admin plan|create --name NAME --max-bytes BYTES --max-objects N [--id ID]");
    println!(
        "       admin create also needs --url URL --parent-token-file FILE --credentials-out FILE"
    );
}
