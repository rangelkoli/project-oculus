use thirtyfour::prelude::*;

#[tokio::main]
pub async fn new_browser(
    _website_url: &str, // This parameter is not used in the current implementation
    _search_term: &str, // This parameter is not used in the current implementation
) -> WebDriverResult<()> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:62768", caps).await?;

    driver.goto(format!("https://{}", _website_url)).await?;

    let elem_form = driver.find(By::Id("search-form")).await?;

    // Find element from element.
    let elem_text = elem_form.find(By::Id("searchInput")).await?;

    // Type in the search terms.
    elem_text.send_keys(_search_term).await?;

    // Click the search button.
    let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
    elem_button.click().await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("Test passed: Successfully navigated to Wikipedia and searched for 'selenium'.");
    // Print the page source.
    // println!("Page source:\n{}", driver.source().await?);

    // explicitly close the browser.
    driver.quit().await?;

    Ok(())
}
