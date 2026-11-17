pub fn name(name: &str) -> &'static str {
    match name {
        "url" | "repo" => "URL",
        "directory" | "file" | "output" | "subdir" | "credentials-out" | "credentials-file"
        | "token-file" | "existing-grant" | "parent-token-file" => "PATH",
        "max-bytes" | "max-objects" | "from" => "NUMBER",
        "parent-grant" => "TOKEN",
        "commit" | "if-sha256" => "SHA256",
        "locations" => "SCOPE",
        _ => "VALUE",
    }
}
