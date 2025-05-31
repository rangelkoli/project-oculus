use std::collections::HashMap;
use thirtyfour::By;
use thirtyfour::prelude::*; // Ensure By is in scope

// Helper function to generate XPath via JavaScript (from previous examples)
// This function is assumed to be robust and is a good fallback.
async fn generate_xpath_via_js(
    driver: &WebDriver,
    element: &WebElement,
) -> WebDriverResult<Option<String>> {
    let script = r#"
        function getElementXPath(element) {
            if (element && element.id) { return '//*[@id="' + element.id + '"]'; } // Prioritize ID
            const paths = [];
            for (; element && element.nodeType == Node.ELEMENT_NODE; element = element.parentNode) {
                let index = 0;
                for (let sibling = element.previousSibling; sibling; sibling = sibling.previousSibling) {
                    if (sibling.nodeType == Node.DOCUMENT_TYPE_NODE) continue;
                    if (sibling.nodeName == element.nodeName) ++index;
                }
                const tagName = element.nodeName.toLowerCase();
                const pathIndex = (index ? "[" + (index + 1) + "]" : ""); // Sibling index
                paths.splice(0, 0, tagName + pathIndex);
            }
            return paths.length ? "/" + paths.join("/") : null;
        }
        return getElementXPath(arguments[0]);
    "#;

    // Convert WebElement to a Value that can be passed to JavaScript
    let args = vec![element.to_json()?];
    let result_json = driver.execute(script, args).await?;

    // Handle ScriptRet return type
    match result_json.json() {
        serde_json::Value::String(s) => Ok(Some(s.clone())),
        serde_json::Value::Null => Ok(None),
        _ => Ok(None), // Fallback for unexpected return types
    }
}

/// Finds all potentially clickable elements on the page and returns a HashMap
/// where keys are descriptive strings and values are `By` locators.
///
/// The function prioritizes locators in this order:
/// 1. `By::Id` if a unique ID attribute is present and verified to be unique.
/// 2. `By::Css` using `tag_name[name='value']` if `name` attribute is present and leads to a unique element.
/// 3. `By::XPath` generated via JavaScript if other strategies fail or are not unique.
/// 4. An EXTREMELY UNRELIABLE basic `By::Css` selector as an absolute last resort.
pub async fn get_all_clickable_element_locators(
    driver: &WebDriver,
) -> WebDriverResult<HashMap<String, By>> {
    let mut clickable_elements_map = HashMap::new();

    let selector = "a[href], button, input[type='button'], input[type='submit'], \
                    input[type='reset'], input[type='image'], input[type='checkbox'], \
                    input[type='radio'], select, textarea, [role='button'], [role='link'], \
                    [role='menuitem'], [role='menuitemcheckbox'], [role='menuitemradio'], \
                    [role='tab'], [role='option'], [role='treeitem'], details > summary, \
                    [contenteditable='true'], [tabindex]:not([tabindex='-1'])";

    let elements: Vec<WebElement> = driver.find_all(By::Css(selector)).await?;

    for (index, element) in elements.iter().enumerate() {
        if !element.is_displayed().await.unwrap_or(false) {
            continue;
        }

        let tag_name = match element.tag_name().await {
            Ok(tn) => tn.to_lowercase(),
            Err(_) => {
                eprintln!(
                    "Warning: Could not get tag name for element (overall index {}). Skipping.",
                    index
                );
                continue;
            }
        };

        let text_content = element.text().await.unwrap_or_default().trim().to_string();

        // Use text content for key generation since accessible_name is not available in thirtyfour
        let descriptive_text_for_key = if !text_content.is_empty() {
            text_content
                .split_whitespace()
                .take(3)
                .collect::<Vec<&str>>()
                .join("_")
        } else {
            String::new() // No descriptive text
        };

        let map_key_base = if !descriptive_text_for_key.is_empty() {
            format!("{}_{}", tag_name, descriptive_text_for_key)
        } else {
            // If no text, use tag name and its original finding index to make the base more distinct
            format!("{}_no_text_idx_{}", tag_name, index)
        };

        // Sanitize key: lowercase, alphanumeric or underscore, no double underscores, trim underscores
        let mut map_key = map_key_base
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<String>()
            .replace("__", "_");
        map_key = map_key.trim_matches('_').to_string();

        // Handle cases where sanitization might result in an empty key (e.g., if base was all non-alphanumeric)
        if map_key.is_empty() {
            map_key = format!("unknown_element_{}", index);
        }

        // Ensure final key uniqueness in the map
        let original_map_key_for_suffixing = map_key.clone();
        let mut key_suffix = 0;
        while clickable_elements_map.contains_key(&map_key) {
            key_suffix += 1;
            map_key = format!("{}_{}", original_map_key_for_suffixing, key_suffix);
        }
        // 'map_key' is now guaranteed to be unique for insertion.

        // --- Determine the best locator ---
        let mut best_locator: Option<By> = None;

        // 1. Try ID first, and verify it's unique on the page
        if let Some(id_val) = element.attr("id").await? {
            if !id_val.is_empty() {
                // Lightweight check for ID uniqueness
                if driver.find_all(By::Id(&id_val)).await?.len() == 1 {
                    best_locator = Some(By::Id(id_val.clone()));
                } else {
                    eprintln!(
                        "  Key: '{}' -> Warning: ID '{}' exists but is not unique. Trying other locators.",
                        map_key, id_val
                    );
                }
            }
        }

        // 2. If no unique ID, try CSS selector by 'name' attribute (common for form elements)
        if best_locator.is_none() {
            if let Some(name_val) = element.attr("name").await? {
                if !name_val.is_empty() {
                    // Escape single quotes for CSS selector: name='value_with_'_apostrophe'
                    let safe_name_val = name_val.replace('\'', "\\'");
                    let css_by_name = format!("{}[name='{}']", tag_name, safe_name_val);
                    // Lightweight check for uniqueness
                    if driver.find_all(By::Css(&css_by_name)).await?.len() == 1 {
                        best_locator = Some(By::Css(css_by_name.clone()));
                    } else {
                        eprintln!(
                            "  Key: '{}' -> Warning: CSS by name attribute '{}' is not unique. Trying XPath.",
                            map_key, css_by_name
                        );
                    }
                }
            }
        }

        // 3. If other specific strategies fail or aren't unique, try generating XPath via JavaScript
        if best_locator.is_none() {
            if let Some(xpath_str) = generate_xpath_via_js(driver, element).await? {
                // Basic check: does the generated XPath find at least one element?
                if !driver.find_all(By::XPath(&xpath_str)).await?.is_empty() {
                    best_locator = Some(By::XPath(xpath_str.clone()));
                } else {
                    eprintln!(
                        "  Key: '{}' -> Warning: JS-generated XPath '{}' found no elements. Skipping this XPath.",
                        map_key, xpath_str
                    );
                }
            }
        }

        // 4. EXTREME FALLBACK: Basic CSS using tag name and the overall index.
        //    This is HIGHLY UNRELIABLE and should generally be avoided or used with extreme caution.
        if best_locator.is_none() {
            let css_very_unreliable_fallback =
                format!("html body {}:nth-of-type({})", tag_name, index + 1);

            eprintln!(
                "  Key: '{}' -> EXTREME FALLBACK (HIGHLY UNRELIABLE) CSS: By::Css('{}'). This locator is very likely to be incorrect or unstable. Consider manually inspecting this element and providing a better locator strategy if possible.",
                map_key, css_very_unreliable_fallback
            );
            best_locator = Some(By::Css(css_very_unreliable_fallback));
        }

        if let Some(locator) = best_locator {
            clickable_elements_map.insert(map_key, locator);
        } else {
            // This case should ideally not be hit if the extreme fallback is always generating *something*.
            // However, if generate_xpath_via_js returned None and we decided not to have an extreme fallback, this is important.
            eprintln!(
                "FATAL WARNING: Could not determine ANY locator for element (tag: '{}', overall_index: {}). This element will NOT be added to the map.",
                tag_name, index
            );
        }
    }

    Ok(clickable_elements_map)
}

/// Alias function for compatibility with existing code
/// Returns the same result as get_all_clickable_element_locators
pub async fn get_interactive_elements_in_hashmap(
    driver: &WebDriver,
) -> WebDriverResult<HashMap<String, By>> {
    get_all_clickable_element_locators(driver).await
}
