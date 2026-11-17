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
    println!("  admin grant --locations 'apps/*' --max-bytes 500000000 --max-objects 15000");
    println!("    --parent-grant CREDENTIALS.json  OR  --root-grant");
    println!("    --root-grant uses SKRYNIA_DATA_DIR (the server's own data directory)");
    println!("    --operations create,modify,list  (default)");
    println!("    --existing-grant CREDENTIALS.json reissues a user token");
    println!("  admin plan|create|inspect|resume|revoke-user|revoke-grant");
    println!("  SKRYNIA_URL supplies --url by default; explicit --url overrides it");
}
