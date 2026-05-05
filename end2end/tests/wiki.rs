use anyhow::Result;
use thirtyfour::prelude::*;

async fn wiki(driver: &WebDriver) -> Result<()> {
    // Navigate to https://wikipedia.org.
    driver.goto("https://wikipedia.org").await?;
    let elem_form = driver.find(By::Id("search-form")).await?;

    // Find element from element.
    let elem_text = elem_form.find(By::Id("searchInput")).await?;

    // Type in the search terms.
    elem_text.send_keys("selenium").await?;

    // Click the search button.
    let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
    elem_button.click().await?;

    // Look for header to implicitly wait for the page to load.
    driver.query(By::ClassName("firstHeading")).first().await?;
    assert_eq!(driver.title().await?, "Selenium - Wikipedia");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let driver = WebDriver::managed(DesiredCapabilities::chrome()).await?;

    let res = wiki(&driver).await;
    match (res, driver.quit().await) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(q)) => Err(q.into()),
        (Err(e), Ok(())) => Err(e),
        (Err(e), Err(_)) => Err(e),
    }
}
