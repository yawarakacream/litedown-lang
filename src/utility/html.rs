use std::collections::HashMap;

use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptions};

use anyhow::Result;

pub struct Html {
    head: Vec<HtmlElement>,
    body: Vec<HtmlElement>,
}
impl Html {
    pub fn new() -> Self {
        Html {
            head: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn append_head(&mut self, element: HtmlElement) {
        self.head.push(element);
    }

    pub fn append_body(&mut self, element: HtmlElement) {
        self.body.push(element);
    }

    pub fn to_string(&self) -> HtmlString {
        let mut head = HtmlElement::new("head");
        for el in &self.head {
            head.append(el.clone());
        }

        let mut body = HtmlElement::new("body");
        for el in &self.body {
            body.append(el.clone());
        }

        HtmlString {
            head: head.to_string(),
            body: body.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct HtmlElement {
    tag: String,
    attr: HashMap<String, String>,
    children: Option<Vec<HtmlElementChild>>,
}

impl HtmlElement {
    pub fn new(tag: &str) -> Self {
        HtmlElement {
            tag: tag.to_string(),
            attr: HashMap::new(),
            children: Some(Vec::new()),
        }
    }

    pub fn new_void(tag: &str) -> Self {
        HtmlElement {
            tag: tag.to_string(),
            attr: HashMap::new(),
            children: None,
        }
    }

    pub fn is_child_empty(&self) -> bool {
        if let Some(children) = &self.children {
            if children.is_empty() {
                return true;
            }
        }
        false
    }

    pub fn set_attr(&mut self, key: &str, value: &str) {
        self.attr.insert(key.to_string(), value.to_string());
    }

    pub fn append(&mut self, element: HtmlElement) -> &mut Self {
        match &mut self.children {
            Some(children) => {
                children.push(HtmlElementChild::HtmlElement(element));
                self
            }
            None => panic!("The void element cannot contain children"),
        }
    }

    pub fn append_raw_text(&mut self, element: &str) -> &mut Self {
        match &mut self.children {
            Some(children) => {
                children.push(HtmlElementChild::String(element.to_string()));
                self
            }
            None => panic!("The void element cannot contain children"),
        }
    }

    pub fn append_text(&mut self, element: &str) -> &mut Self {
        self.append_raw_text(&escape_html_text(element))
    }

    fn write_to_string(&self, buffer: &mut String) {
        buffer.push('<');
        buffer.push_str(&self.tag);
        for (k, v) in &self.attr {
            buffer.push_str(&format!(" {k}={v:?}"));
        }
        buffer.push('>');

        if let Some(children) = &self.children {
            for child in children {
                match child {
                    HtmlElementChild::String(string) => buffer.push_str(&string),
                    HtmlElementChild::HtmlElement(el) => el.write_to_string(buffer),
                }
            }
            buffer.push_str(&format!("</{}>", self.tag));
        }
    }
}

impl ToString for HtmlElement {
    fn to_string(&self) -> String {
        let mut buffer = String::new();
        self.write_to_string(&mut buffer);
        buffer
    }
}

#[derive(Clone)]
enum HtmlElementChild {
    String(String),
    HtmlElement(HtmlElement),
}

fn escape_html_text(str: &str) -> String {
    let mut buffer = String::new();
    for c in str.chars() {
        match c {
            ' ' => buffer.push_str("&nbsp;"),
            '<' => buffer.push_str("&lt;"),
            '>' => buffer.push_str("&gt;"),
            '&' => buffer.push_str("&amp;"),
            '"' => buffer.push_str("&quot;"),
            '\n' => buffer.push_str("<br>"),
            _ => buffer.push(c),
        }
    }
    buffer
}

pub fn print_html_to_pdf(html_path: &str) -> Result<Vec<u8>> {
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .build()
            .expect("Could not find chrome-executable"),
    )?;

    let tab = browser.new_tab().unwrap();

    let mut pdf_option = PrintToPdfOptions::default();
    pdf_option.margin_top = Some(0.0);
    pdf_option.margin_bottom = Some(0.0);
    pdf_option.margin_left = Some(0.0);
    pdf_option.margin_right = Some(0.0);
    pdf_option.scale = Some(1.0);
    pdf_option.prefer_css_page_size = Some(true);
    pdf_option.print_background = Some(true);

    tab.navigate_to(&format!("file://{}", html_path))?
        .wait_until_navigated()?
        .print_to_pdf(Some(pdf_option))
}

pub struct HtmlString {
    head: String,
    body: String,
}

impl HtmlString {
    pub fn get_head(&self) -> String {
        self.head.clone()
    }

    pub fn get_body(&self) -> String {
        self.body.clone()
    }

    pub fn merge(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str("<!DOCTYPE html>");
        buffer.push_str("<html lang=\"ja\">");
        buffer.push_str(&self.head.clone());
        buffer.push_str(&self.body.clone());
        buffer.push_str("</html>");
        buffer
    }
}
