# Advanced browser automation with security bypass techniques
# This script handles authentication for the US visa appointment system
# which has multiple layers of protection against automation

from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.firefox.service import Service
import sys
import time
import os

def login(url, username, password, proxy_address):
    # Set up Firefox options with proxy configuration
    firefox_options = Options()
    
    # Enable headless mode for unattended operation
    firefox_options.add_argument('--headless')
    
    # Configure proxy with proper security settings
    proxy_ip, proxy_port = proxy_address.split(':')
    firefox_options.set_preference("network.proxy.type", 1)
    firefox_options.set_preference("network.proxy.http", proxy_ip)
    firefox_options.set_preference("network.proxy.http_port", int(proxy_port))
    firefox_options.set_preference("network.proxy.ssl", proxy_ip)
    firefox_options.set_preference("network.proxy.ssl_port", int(proxy_port))
    
    # Disable proxy authentication dialogues that could block automation
    firefox_options.set_preference("network.proxy.socks_remote_dns", True)
    firefox_options.set_preference("network.auth.use-sspi", False)
    firefox_options.set_preference("signon.autologin.proxy", False)
    
    # Accept insecure certificates to handle some proxy situations
    firefox_options.set_capability("acceptInsecureCerts", True)
    firefox_options.set_capability("proxy", {
        'httpProxy': proxy_address,
        'sslProxy': proxy_address,
        'proxyType': 'manual',
    })

    # Initialize the Firefox driver with silent logging
    service = Service(log_path=os.devnull)
    driver = webdriver.Firefox(options=firefox_options, service=service)
    driver.set_window_size(1920, 1080)  # Set large window to avoid mobile layouts

    try:
        # Navigate to login page
        driver.get(url)

        # Wait for the email field with explicit wait for reliability
        email_field = WebDriverWait(driver, 10).until(
            EC.visibility_of_element_located((By.ID, "user_email"))
        )
        email_field.send_keys(username)

        # Enter password
        password_field = driver.find_element(By.ID, "user_password")
        password_field.send_keys(password)

        # Handle the policy confirmation checkbox with multiple fallback methods
        # This is a common anti-automation measure that requires special handling
        checkbox = WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.ID, "policy_confirmed"))
        )

        # Scroll the checkbox into view before attempting interaction
        driver.execute_script("arguments[0].scrollIntoView({block: 'center'});", checkbox)
        time.sleep(1)  # Allow time for any animations to complete

        # Try multiple click methods in case one fails
        try:
            # Method 1: Direct click
            checkbox.click()
        except Exception:
            try:
                # Method 2: JavaScript click
                driver.execute_script("arguments[0].click();", checkbox)
            except Exception:
                # Method 3: Click the label instead of checkbox directly
                label = driver.find_element(By.XPATH, "//label[@for='policy_confirmed']")
                label.click()

        # Verify if the checkbox is checked via JavaScript
        is_checked = driver.execute_script("return arguments[0].checked;", checkbox)
        if not is_checked:
            print("Warning: Checkbox may not be checked. Please verify manually.")

        # Find and click the submit button
        submit_button = WebDriverWait(driver, 10).until(
            EC.element_to_be_clickable((By.XPATH, "//input[@type='submit' and @value='Oturum Aç']"))
        )
        submit_button.click()

        # Wait for cookies to be set
        time.sleep(5)

        # Extract the yatri_session cookie which is needed for authentication
        yatri_session_cookie = None
        for cookie in driver.get_cookies():
            if "_yatri_session" in cookie['name']:
                yatri_session_cookie = cookie
                break

        if yatri_session_cookie:
            print(f"{yatri_session_cookie['value']}")
        else:
            print("Error.")

    except Exception as e:
        print("Error.")
    finally:
        driver.quit()
