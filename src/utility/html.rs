use std::collections::HashMap;

use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptions};

// #[macro_export]
// macro_rules! html {
//     (<$tag:ident $($attrkey:ident={$attrvalue:expr})*>) => {
//         concat!("<", stringify!($tag), $(" ", stringify!($attrkey), "=\"", $attrvalue, "\"", )* ">")
//     };

//     (</$tag:ident>) => {
//         concat!("</", stringify!($tag), ">")
//     };
// }

#[macro_export]
macro_rules! attrs {
    ($($name:expr => $value:expr),*) => {
        vec![ $( ($name, $value) ),* ].into_iter().collect()
    };
}

pub struct HtmlWriter {
    buffer: String,
    stack: Vec<String>,
    level: usize,
}

impl HtmlWriter {
    pub fn new() -> Self {
        let mut instance = HtmlWriter {
            buffer: String::new(),
            stack: Vec::new(),
            level: 0,
        };
        instance.init().unwrap();
        instance
    }

    fn init(&mut self) -> Result<(), String> {
        self.buffer.push_str("<!DOCTYPE html>");
        self.open_element("html", attrs! {"lang" => "ja"})?;

        self.open_element("head", attrs! {})?;
        self.add_void_element("meta", attrs! {"charset" => "UTF-8"})?;
        self.add_void_element(
            "meta",
            attrs! {"http-equiv" => "X-UA-Compatible", "content" => "IE=edge"},
        )?;
        self.add_void_element(
            "meta",
            attrs! {"name" => "viewport", "content" => "width=device-width, initial-scale=1.0"},
        )?;

        self.add_inline_element("title", attrs! {}, "litedown")?;
        self.close_element("head")?;

        self.buffer.push('\n');

        self.open_element("body", attrs! {})?;
        Ok(())
    }

    fn write_open_tag(&mut self, tag: &str, attrs: HashMap<&str, &str>) {
        self.buffer.push_str("<");
        self.buffer.push_str(tag);

        for (k, v) in attrs.iter() {
            self.buffer.push(' ');
            self.buffer.push_str(&format!("{}={:?}", k, v));
        }

        self.buffer.push_str(">");
    }

    fn write_close_tag(&mut self, tag: &str) {
        self.buffer.push_str("</");
        self.buffer.push_str(tag);
        self.buffer.push_str(">");
    }

    pub fn open_element(&mut self, tag: &str, attrs: HashMap<&str, &str>) -> Result<(), String> {
        self.stack.push(String::from(tag));

        self.buffer.push('\n');
        // self.buffer.push_str(&"  ".repeat(self.level));

        self.write_open_tag(tag, attrs);
        self.level += 1;
        Ok(())
    }

    pub fn write_inner(&mut self, text: &str) -> Result<(), String> {
        if self.level == 0 {
            return Err("No tag".to_string());
        }
        self.buffer.push_str(&escape_html_text(text));
        Ok(())
    }

    pub fn write_raw_inner(&mut self, text: &str) -> Result<(), String> {
        if self.level == 0 {
            return Err("No tag".to_string());
        }
        self.buffer.push_str(text);
        Ok(())
    }

    pub fn close_element(&mut self, tag: &str) -> Result<(), String> {
        self.level -= 1;
        match self.stack.pop() {
            Some(to_be_closed) => {
                if to_be_closed != tag {
                    return Err(format!("Illegal tag: {}, expected: {}", tag, to_be_closed));
                }
                self.buffer.push('\n');
                // self.buffer.push_str(&"  ".repeat(self.level));
                self.write_close_tag(tag);
                Ok(())
            }
            None => Err("All tags have already been closed".to_string()),
        }
    }

    pub fn add_inline_element(
        &mut self,
        tag: &str,
        attrs: HashMap<&str, &str>,
        inner_text: &str,
    ) -> Result<(), String> {
        self.write_open_tag(tag, attrs);
        self.write_inner(inner_text).unwrap();
        self.write_close_tag(tag);
        Ok(())
    }

    pub fn add_void_element(
        &mut self,
        tag: &str,
        attrs: HashMap<&str, &str>,
    ) -> Result<(), String> {
        self.open_element(tag, attrs)?;
        self.level -= 1;
        self.stack.pop();
        Ok(())
    }

    pub fn build(mut self) -> Result<String, String> {
        self.close_element("body")?;
        self.close_element("html")?;
        if self.level != 0 || self.stack.len() != 0 {
            return Err("Illegal state".to_string());
        }
        Ok(self.buffer)
    }
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

pub fn print_html_to_pdf(html_path: &str) -> anyhow::Result<Vec<u8>> {
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
