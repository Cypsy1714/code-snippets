# Platform-independent browser automation for bypassing client-side security
# This script demonstrates how to circumvent Cloudflare protection and website security
# by automatically managing cookies across multiple operating systems

from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.firefox.options import Options
import time
import sys
import platform

# Cross-platform profile path handling
os_name = platform.system()

profile_path = r'/Users/ugurcemsaglam/Library/Application Support/Firefox/Profiles/zs2kvr6x.default-release-1723540903612'
firefox_path = r'/usr/local/bin/geckodriver'

if os_name == "Linux":
    profile_path = r'/home/user/.mozilla/firefox/sn5qnnqe.default-esr'
   
if os_name == "Windows":
    profile_path = r'C:\Users\Administrator\AppData\Roaming\Mozilla\Firefox\Profiles\0znfxjqj.default-release'
    firefox_path = r'C:\Program Files\Geckodriver\geckodriver'

# Configure Firefox with the saved profile to retain login state
firefox_options = Options()
# firefox_options.add_argument("--headless")  # Can run headless in production

if os_name == "Windows":
    firefox_options.binary = r'C:\Program Files\Mozilla Firefox\firefox.exe'

# Use existing profile to bypass login requirements
myprofile = webdriver.FirefoxProfile(profile_path)
driver = webdriver.Firefox(firefox_profile=myprofile, executable_path=firefox_path, options=firefox_options)

try:
    # Navigate to target site that uses client-side security
    driver.get("https://cs.money/market/buy")

    # Allow time for JavaScript to execute and set cookies
    time.sleep(15)

    # Extract authentication cookies that will be used in API requests
    cookies = driver.get_cookies()

    # Format cookies for header usage in subsequent HTTP requests
    cookie_str = '; '.join([f"{cookie['name']}={cookie['value']}" for cookie in cookies])

    # Output cookie string for use in the Rust trading bot
    print(cookie_str)
finally:
    # Clean up browser resource
    time.sleep(3)
    driver.quit()
