use playwright::Playwright;
use thirtyfour::prelude::*;

#[tokio::main]
pub async fn create_new_browser() -> Result<playwright::api::Browser, playwright::Error> {
    let playwright = Playwright::initialize().await?;
    let chromium = playwright.chromium();
    let browser = chromium
        .connect_over_cdp_builder("http://localhost:4444")
        .connect_over_cdp()
        .await?;

    Ok(browser)
}
