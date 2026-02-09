use super::*;
use web_sys::wasm_bindgen::JsCast;

pub fn render_katex() {
    let document = gloo::utils::document();
    let ctx = katex::KatexContext::default();
    let settings = katex::Settings::builder()
        .display_mode(false)
        .output(katex::OutputFormat::Mathml)
        .build();

    let list = document.query_selector_all("span.math").unwrap();

    tracing::info!("get {} katex to render", list.length());

    for i in 0..list.length() {
        let node = list.item(i).unwrap();
        let elm: web_sys::Element = node.dyn_into().unwrap();
        let inner = elm.inner_html();
        tracing::info!("render katex {}", inner);
        let ret = match katex::render_to_string(&ctx, &inner, &settings) {
            Ok(ret) => ret,
            Err(err) => {
                tracing::error!("katex render error {:#?}", err);
                inner
            }
        };
        tracing::info!("render result {}", ret);
        elm.set_inner_html(&ret);
    }
}

pub fn render_code_block() {
    let mut listeners = use_signal(Vec::new);
    listeners.clear();
    let document = gloo::utils::document();
    let list = document.query_selector_all("div.markdown code").unwrap();
    for i in 0..list.length() {
        let node = list.item(i).unwrap();
        let elm: web_sys::Element = node.dyn_into().unwrap();
        let inner = elm.inner_html();
        let button = document.create_element("button").unwrap();
        button.set_inner_html("copy");
        let listener = gloo::events::EventListener::new(&button, "click", move |_| {
            let _ = web_sys::window()
                .unwrap()
                .navigator()
                .clipboard()
                .write_text(&inner);
        });
        listeners.push(listener);
        elm.insert_adjacent_element("beforebegin", &button).unwrap();
    }
}

#[component]
pub fn Markdown(content: String) -> Element {
    use pulldown_cmark::Options;
    let options = Options::ENABLE_MATH;
    let parser = pulldown_cmark::Parser::new_ext(&content, options);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    tracing::info!("{}", &html);

    use_effect(|| {
        spawn(async {});
        render_code_block();
        render_katex();
    });

    rsx! {
        div {class:"markdown", dangerous_inner_html: html }
    }
}
