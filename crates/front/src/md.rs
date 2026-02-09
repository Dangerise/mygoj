use super::*;

#[component]
pub fn Markdown(content: String) -> Element {
    let parser = pulldown_cmark::Parser::new(&content);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    rsx! {
        div { dangerous_inner_html: html }
    }
}
