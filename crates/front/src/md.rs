use super::*;

#[component]
pub fn Markdown(md: String) -> Element {
    let parser = pulldown_cmark::Parser::new(&md);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    rsx! {
        div { dangerous_inner_html: html }
    }
}
