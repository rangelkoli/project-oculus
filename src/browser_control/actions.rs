use thirtyfour::prelude::*;

pub async fn go_to_url(driver: &WebDriver, url: &str) -> WebDriverResult<()> {
    print!("Navigating to URL: {}", url);
    driver.new_tab().await?;
    driver.goto(url).await?;
    Ok(())
}

pub async fn extract_content(driver: &WebDriver) -> WebDriverResult<String> {
    print!("Extracting content from the current page.");
    let content = driver.find(By::Tag("body")).await?.text().await?;
    Ok(content)
}

pub async fn click_element(driver: &WebDriver, selector: &str) -> WebDriverResult<()> {
    print!("Clicking element with selector: {}", selector);
    let element = driver.find(By::Css(selector)).await?;
    element.click().await?;
    Ok(())
}
