pub fn print() {
    println!("skrynia-tools: HTTP-only application release management");
    println!("  app deploy --url URL --name NAME --directory BUILD [--commit SHA]");
    println!("  app deploy --url URL --name NAME --repo GITHUB_URL --commit SHA --subdir PATH");
    println!("  app list|ls --url URL");
    println!("  app versions|current --url URL --name NAME");
    println!("  app show|rollback|verify --url URL --name NAME --release ID");
    println!("  app compare|diff --url URL --name NAME --from ID --to ID");
    println!("  app verify-all --url URL --name NAME");
    println!("  app new --name NAME --directory PATH");
    println!("  Authentication: SKRYNIA_TOKEN, or --token-env VARIABLE");
    println!("  Rollback CAS-replaces one pointer, O(1) independent of history or assets");
    println!("  app deploy builds on this machine (never on Skrynia)");
    println!("  object get|create|replace|delete|list --url URL --key KEY");
    println!("  log init|append|read --prefix PREFIX --url URL");
    println!("  admin plan|create|inspect|resume|revoke-user|revoke-grant");
}
