pub fn build() -> Result<reqwest::Client, reqwest::Error> {
    // The private Program0 bearer travels in the POST body. Never route
    // it through inherited proxies or follow an untrusted HTTP redirect.
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
}
