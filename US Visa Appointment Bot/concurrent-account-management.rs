// This system manages 100+ concurrent US visa appointment checking sessions
// With thread-safe shared state and inter-thread communication

pub async fn update_cookies(accounts: Arc<RwLock<Vec<Account>>>) {
    // Update all the cookies once first
    let account_data = {
        let accounts_guard = accounts.read().await;
        // Clone minimal data needed to avoid holding the lock during network operations
        accounts_guard
            .iter()
            .map(|acc| (
                acc.username.clone(),
                acc.password.clone(),
                acc.proxy_string.clone(),
            ))
            .collect::<Vec<_>>()
    }; // Read lock released here
    
    // Generate new cookies concurrently
    let cookie_futures = account_data
        .iter()
        .map(|(username, password, proxy_string)| {
            get_cookie(
                username.to_string(),
                password.to_string(),
                proxy_string.to_string(),
            )
        });
    
    // Await all futures simultaneously for maximum efficiency
    let new_cookies = join_all(cookie_futures).await;

    // Update all cookies with a single write lock
    {
        let mut accounts_guard = accounts.write().await;
        for (i, cookie_result) in new_cookies.into_iter().enumerate() {
            if let Ok(new_cookie) = cookie_result {
                if i < accounts_guard.len() {
                    accounts_guard[i].cookie = new_cookie;
                }
            }
        }
    }

    // Continuous cookie refresh loop
    let mut account_i = 1;
    loop {
        thread::sleep(Duration::from_secs(COOKIE_UPDATE_COOLDOWN as u64));

        // Use read lock to access account data
        let accounts_guard = accounts.read().await;
        let account = &accounts_guard[account_i];

        // Clone data to avoid holding the lock during async operation
        let username = account.username.clone();
        let password = account.password.clone();
        let proxy = account.proxy_string.clone();

        // Drop the lock before the async operation
        drop(accounts_guard);

        let cookie = get_cookie(username.clone(), password, proxy).await;

        if let Ok(cookie_str) = cookie {
            // Now acquire a write lock to update the cookie
            {
                let mut accounts_guard = accounts.write().await;
                // Update the cookie for the specific account
                accounts_guard[account_i].cookie = cookie_str;
                // Write lock is automatically released when block ends
            }
        } else if let Err(err) = cookie {
            let string = format!(
                "Error when updating cookie. Username: {:?}\nError: {:?}",
                username, err
            );
            print_n_log(string, true, true);
        }

        // Rotate to next account
        account_i += 1;
        if account_i == ACCOUNT_COUNT as usize {
            account_i = 0;
        }
    }
}

// Main function to check appointments using multiple accounts in a controlled manner
pub async fn check_appointments(accounts: Arc<RwLock<Vec<Account>>>) {
    let mut account_i = 0;
    let mut cycle_i = 0;
    let mut batch_i = 0;

    loop {
        // Get read lock to access the vector
        let accounts_guard = accounts.read().await;
        let account = &accounts_guard[account_i as usize];

        // Clone minimal data to avoid holding the lock
        let username = account.username.clone();
        let user_payment_id = account.user_payment_id.clone();
        let cookie = account.cookie.clone();
        let proxy = account.proxy_string.clone();

        // Drop the lock before the async operation
        drop(accounts_guard);

        // Make the API request with the cloned data
        let appointments_ =
            scrape_with_account_data(username, user_payment_id, proxy, cookie).await;

        // Process the results
        if let Ok(appointments) = appointments_ {
            print_n_log(format!("Success: {:?}", appointments), true, true);
            update_appointments(appointments).await;
        } else {
            let err_msg = appointments_.unwrap_err();
            print_n_log(
                format!("Error! Cannot get appointment data. \n {}", err_msg),
                true, 
                true,
            );
        }

        // Rate limiting between requests
        thread::sleep(Duration::from_secs(COOLDOWN_PER_REQUEST as u64));

        // Advanced rotation strategy to manage account usage patterns
        find_next_account_i(&mut account_i, &mut cycle_i, &mut batch_i);
    }
}

// Complex rotation logic to distribute load across accounts in a pattern
// that minimizes detection risk while maximizing appointment checking
fn find_next_account_i(account_i: &mut i32, cycle_i: &mut i32, batch_i: &mut i32) {
    let start_i = *batch_i * ACCOUNTS_PER_BATCH;
    let end_i = start_i + ACCOUNTS_PER_BATCH - 1;

    // If the account_i is the last in current batch
    if *account_i == end_i {
        // If we have reached the max cycle amount, go to the next batch
        if *cycle_i == CYCLE_PER_BATCH {
            // Check if we have enough accounts to move forward one batch
            if (*batch_i + 2) * ACCOUNTS_PER_BATCH > ACCOUNT_COUNT {
                *batch_i = 0;
                *cycle_i = 0;
                *account_i = 0;
            } else {
                *batch_i += 1;
                *cycle_i = 0;
                *account_i = *batch_i * ACCOUNTS_PER_BATCH;
            }
        } else {
            // Reset the account_i to start_i and add one to cycle_i
            *account_i = start_i;
            *cycle_i += 1;
        }
    } else {
        *account_i += 1;
    }
}
