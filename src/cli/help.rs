pub fn print() {
    println!("skrynia-tools — GitHub app releases and optional Skrynia data");
    println!();
    println!("App code (independent of application data protocols):");
    println!(
        "  app deploy --name NAME --repo GITHUB_URL --subdir PATH --apps-dir DIR [--commit SHA]"
    );
    println!("  app list --apps-dir DIR [--format table|json]");
    println!("  app doctor --apps-dir DIR [--format table|json]");
    println!("  app versions --name NAME --apps-dir DIR [--format table|json]");
    println!("  app current --name NAME --apps-dir DIR");
    println!("  app compare --name NAME --from ID --to ID --apps-dir DIR");
    println!("  app show --name NAME --release ID --apps-dir DIR");
    println!("  app verify --name NAME [--release ID] --apps-dir DIR");
    println!("  app rollback --name NAME --release ID --apps-dir DIR");
    println!("  app new --name NAME --directory DIR");
    println!("  Remote app commands: --ssh-host HOST --ssh-user USER");
    println!("      --ssh-key PRIVATE_KEY --ssh-known-hosts HOSTKEY_FILE [--ssh-port PORT]");
    println!("  app releases|inspect|history|activate (compatibility aliases)");
    println!("  Rollback changes only the code symlink; constant work, no migrations.");
    println!();
    println!("Optional Skrynia storage:");
    println!("  object get|create|replace|delete|list --url URL --key KEY");
    println!("  log init|append|read --prefix PREFIX --url URL");
    println!("  admin plan|create --name NAME --max-bytes BYTES --max-objects N");
    println!("  admin inspect|resume --credentials-file PRIVATE_FILE [--url URL]");
    println!("      resume requires explicit acknowledgment if issuance outcome is unknown");
    println!("  admin revoke-user|revoke-grant --credentials-file FILE --url URL");
    println!("      --confirm-grant-id ID [--retry-after-uncertain true]");
}
