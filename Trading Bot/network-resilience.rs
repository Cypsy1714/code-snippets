//
// Advanced network resilience with proxy rotation, retry logic, and error handling
//

// Generic retry function for network operations
// Handles transient failures with exponential backoff
#[allow(dead_code)]
async fn repeat_async_function<'a, F, Fut, T, E>(
    mut f: F,
    max_retries: u32,
    input: &'a str,
) -> Result<T, E>
where
    F: FnMut(&'a str) -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    while attempts < max_retries {
        match f(input).await {
            Ok(result) => return Ok(result),
            Err(_) if attempts < max_retries - 1 => {
                attempts += 1;
                // Sleep between retries with increasing duration
                sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
    // If all attempts failed, execute one more time to return the actual error
    f(input).await
}

// Proxy rotation and request resilience
async fn send_request_with_proxy_and_timeout_and_retry(
    url: &str,
    proxy_url: &str,
    headers: HeaderMap,
    body: String,
    username: &str,
    password: &str,
    timeout_secs: u64,
    max_retries: usize,
) -> Result<reqwest::Response, reqwest::Error> {
    // Configure proxy authentication
    let proxy = Proxy::all(proxy_url)
        .unwrap()
        .basic_auth(username, password);
    
    // Build client with proxy and timeout
    let client = Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;

    let mut attempts = 0;

    loop {
        attempts += 1;
        match client
            .post(url)
            .headers(headers.clone())
            .body(body.clone())
            .send()
            .await
        {
            Ok(response) => {
                return Ok(response);
            }
            Err(_) if attempts <= max_retries => {
                // Wait before retry with linear backoff
                sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

// API key rotation to avoid rate limiting
fn get_scrape_key() -> String {
    // Rotate between multiple API keys to avoid detection and rate limits
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..=3);
    SCRAPE_KEYS[random_number].to_string()
}

// Example of a market API request with full security measures
pub async fn get_item_price(
    market_hash_name: String,
    max_trade_hold: i32,
) -> Result<reqwest::Response, reqwest::Error> {
    // Start performance tracking
    let start = SystemTime::now();

    // Determine item category for correct API filtering
    let mut category = "1";
    if market_hash_name.contains("StatTrak") {
        category = "3";
    }
    if market_hash_name.contains("Souvenir") {
        category = "5";
    }

    // Construct the API request
    let url = "https://api.bitskins.com/market/search/730";
    let json_str = format!(
        r#"{{"order":[{{"field":"price","order":"ASC"}}],"offset":0,"limit":30,"where":{{"skin_name":"{}","tradehold_to":{},"price_from":10,"price_to":25000000,"category_id":[{}]}}}}"#,
        market_hash_name, max_trade_hold, category
    );

    // Set appropriate headers
    let mut header = reqwest::header::HeaderMap::new();
    header.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str("application/json").unwrap(),
    );

    // Get proxy from rotation pool
    let proxy_data = data::get_proxy(Market::BitSkins);

    // Send request with full resilience stack
    let body = send_request_with_proxy_and_timeout_and_retry(
        url,
        &proxy_data.0,
        header.clone(),
        json_str.clone(),
        &proxy_data.1,
        &proxy_data.2,
        15,  // 15 second timeout
        0,   // No retries for this specific endpoint
    )
    .await;

    // Log performance metrics
    let after = SystemTime::now();
    let passed = after.duration_since(start).unwrap();
    let log_txt = format!(
        "bitskins_api | get_item_price(market_hash_name: {}) | The HTTP request took {:?}.\n",
        market_hash_name, passed
    );
    log_write(&log_txt);
    
    body
}
