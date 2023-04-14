use anyhow::Result;

use crate::utility::html::HtmlElement;

pub fn create_slide_page<E>(mut el: E) -> Result<HtmlElement>
where
    E: FnMut(&mut HtmlElement) -> Result<()>,
{
    let mut page = HtmlElement::new("div");
    page.set_attr("class", "page");
    el(&mut page)?;

    let mut container = HtmlElement::new("div");
    container.set_attr("class", "page-container");
    container.append(page);
    Ok(container)
}
