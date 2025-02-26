// Static indices to track which proxy is used for each market
// Using separate indices prevents detection patterns that could emerge
// from using the same proxy rotation sequence across all markets
static mut MARKETCSGO_NUM: usize = 0;
static mut DMARKET_NUM: usize = 0;
static mut CSMONEY_NUM: usize = 0;
static mut CSFLOAT_NUM: usize = 0;
static mut BITSKINS_NUM: usize = 0;
static mut WAXPEER_NUM: usize = 0;

// Returns a proxy from the list according to the market
// Each market has its own rotation index to avoid correlation
// (proxy_url, proxy_username, proxy_password)
pub fn get_proxy(market: Market) -> (String, String, String) {
    let proxies = vec![
        "45.86.48.213:50100",
        "45.86.50.46:50100",
        "45.86.49.52:50100",
        "45.86.50.11:50100",
        "45.86.48.9:50100",
        "45.86.50.186:50100",
        "45.86.48.7:50100",
        "45.86.50.124:50100",
        "45.86.50.128:50100",
        "45.86.50.63:50100",
    ];
    let username = "ugurcemsaglam";
    let password = "UsNEtD8J8c";
    let mut proxy_url = "";
    
    // UNSAFE block used in a controlled way
    // This is safe because the only code accessing these variables is this function,
    // which ensures proper increment/bounds checking behavior
    unsafe {
        match market {
            Market::Steam => {},  // No proxy needed for Steam
            Market::Buff => {},   // No proxy needed for Buff
            Market::LisSkins => {},  // No proxy needed for LisSkins
            
            // Each market has its own index counter to create different rotation patterns
            Market::MarketCSGO => {
                proxy_url = proxies[MARKETCSGO_NUM];
                MARKETCSGO_NUM += 1;
                if MARKETCSGO_NUM >= proxies.len() {
                    MARKETCSGO_NUM = 0;
                }
            }
            Market::DMarket => {
                proxy_url = proxies[DMARKET_NUM];
                DMARKET_NUM += 1;
                if DMARKET_NUM >= proxies.len() {
                    DMARKET_NUM = 0;
                }
            }
            Market::CSMoney => {
                proxy_url = proxies[CSMONEY_NUM];
                CSMONEY_NUM += 1;
                if CSMONEY_NUM >= proxies.len() {
                    CSMONEY_NUM = 0;
                }
            }
            Market::CSFloat => {
                proxy_url = proxies[CSFLOAT_NUM];
                CSFLOAT_NUM += 1;
                if CSFLOAT_NUM >= proxies.len() {
                    CSFLOAT_NUM = 0;
                }
            }
            Market::BitSkins => {
                proxy_url = proxies[BITSKINS_NUM];
                BITSKINS_NUM += 1;
                if BITSKINS_NUM >= proxies.len() {
                    BITSKINS_NUM = 0;
                }
            }
            Market::WaxPeer => {
                proxy_url = proxies[WAXPEER_NUM];
                WAXPEER_NUM += 1;
                if WAXPEER_NUM >= proxies.len() {
                    WAXPEER_NUM = 0;
                }
            }
        }
    }
    
    // Return the proxy details for the requester
    (
        format!("{}", proxy_url),
        username.to_string(),
        password.to_string(),
    )
}
