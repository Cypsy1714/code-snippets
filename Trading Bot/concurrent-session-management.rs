// Efficient concurrent processing of multiple market queries
// This function demonstrates how the trading bot manages 100+ concurrent sessions
// by controlling the parallel request flow with futures

async fn get_all_prices(map: &mut HashMap<String, Item>) {
    let mut to_do = Vec::new();
    let mut amount = 0;
    
    for (_key, value) in map {
        amount += 1;
        // Push the async operation to our queue without awaiting
        to_do.push(value.get_all_prices());
        
        // Once we hit our parallelism limit, execute all requests concurrently
        if amount >= PARALLEL_REQUESTS {
            // Join all futures and await their completion
            futures::future::join_all(to_do).await;
            to_do = Vec::new();
            amount = 0;
            
            // Rate limiting - prevent overwhelming the target servers
            let wait_time = time::Duration::from_secs(1);
            thread::sleep(wait_time);
        }
    }
    
    // Process any remaining requests
    futures::future::join_all(to_do).await;
}

// Implementation of market-specific price retrieval with controlled concurrency
async fn get_given_prices(map: &mut HashMap<String, Item>, markets: Vec<Market>) {
    let mut to_do = Vec::new();
    let mut amount = 0;
    
    for (_key, value) in map {
        amount += 1;
        // Clone the markets vector for each item to avoid ownership issues
        to_do.push(value.get_given_prices(markets.clone()));
        
        if amount >= PARALLEL_REQUESTS {
            futures::future::join_all(to_do).await;
            to_do = Vec::new();
            amount = 0;
            let wait_time = time::Duration::from_secs(1);
            thread::sleep(wait_time);
        }
    }
    
    futures::future::join_all(to_do).await;
}

// Trait implementation showing how each Item manages its own concurrent market data fetching
#[allow(async_fn_in_trait)]
pub trait MarketFunctions {
    async fn get_all_prices(&mut self);
    async fn get_given_prices(&mut self, markets: Vec<Market>);
    // Other functions omitted for brevity
}

impl MarketFunctions for Item {
    // Gets the price of the item in all the markets and updates self
    // Using multithreading with futures for performance
    async fn get_all_prices(&mut self) {
        let timeout_duration = tokio::time::Duration::from_secs(100);
        let prices_vec = self.price.clone();
        
        // Use timeout to prevent hanging on slow requests
        let prices_t = timeout(
            timeout_duration, 
            get_all_prices_request(self.name.clone(), prices_vec)
        ).await;
        
        self.price = Vec::new();
        if let Ok(prices) = prices_t {
            for price in prices {
                if let Ok(val) = price.1 {
                    self.price.push(val);
                } else {
                    log_functions::log_err(&price.1.unwrap_err());
                }
            }
        } else {
            log_functions::log_err("The price timeout has triggered.");
        }
    }

    // Other functions omitted for brevity
}
