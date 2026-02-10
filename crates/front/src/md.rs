use super::*;
use std::collections::BTreeSet;
use std::sync::Arc;
use web_sys::wasm_bindgen::JsCast;

fn get_tab_width(code: &str) -> usize {
    let mut set = BTreeSet::new();
    for line in code.lines() {
        if line.is_empty() {
            continue;
        }
        let first = line.chars().next().unwrap();
        if first == '\t' {
            continue;
        }
        let spaces_len = line.chars().take_while(|&c| c == ' ').count();
        if spaces_len == 0 {
            continue;
        }
        set.insert(spaces_len);
    }
    let Some(&smallest) = set.first() else {
        return 4;
    };
    for i in (1..=smallest).rev() {
        if set.iter().all(|x| x % i == 0) {
            return i;
        }
    }
    unreachable!()
}

pub fn render_code(lang: &str, code: &str) -> Option<String> {
    let tab_widht = get_tab_width(code);
    let Some(mut highlighter) = synoptic::from_extension(lang, tab_widht) else {
        return None;
    };
    let lines = code.lines().map(str::to_string).collect::<Vec<_>>();
    highlighter.run(&lines);
    let mut html = String::with_capacity(code.len() * 7);
    for (number, line) in lines.into_iter().enumerate() {
        for token in highlighter.line(number, &line) {
            use synoptic::TokOpt;
            match token {
                TokOpt::Some(text, kind) => {
                    let color = match kind.as_str() {
                        "comment" => "gray",
                        "attribute" => "lightgray",
                        "digit" => "navy",
                        "reference" => "indigo",
                        "string" => "green",
                        "macros" => "maroon",
                        "macro" => "fuchsia",
                        "boolean" => "blue",
                        "character" => "royalblue",
                        "keyword" => "purple",
                        "struct" => "violet",
                        "operator" => "darkviolet",
                        "namespace" => "orchid",
                        "function" => "red",
                        _ => panic!("get unknown token kind {kind}"),
                    };
                    html.push_str(&format!("<span style=\"color:{color}\">{text}</span>"));
                }
                TokOpt::None(text) => {
                    html.push_str(&text);
                }
            }
        }
        html.push('\n');
    }
    Some(html)
}

pub fn render_katex() {
    let document = gloo::utils::document();
    let ctx = katex::KatexContext::default();
    let settings = katex::Settings::builder()
        .display_mode(false)
        .output(katex::OutputFormat::Mathml)
        .build();

    let list = document.query_selector_all("span.math").unwrap();

    for i in 0..list.length() {
        let node = list.item(i).unwrap();
        let elm: web_sys::Element = node.dyn_into().unwrap();
        let inner = elm.inner_html();
        let inner = inner.replace("&amp;", "&");
        let ret = match katex::render_to_string(&ctx, &inner, &settings) {
            Ok(ret) => ret,
            Err(err) => {
                tracing::error!("katex render error {:#?}", err);
                inner
            }
        };
        elm.set_inner_html(&ret);
    }
}

pub fn render_code_block() {
    let mut listeners = use_signal(Vec::new);
    listeners.clear();
    let document = gloo::utils::document();
    let list = document
        .query_selector_all("div.markdown pre code")
        .unwrap();

    for i in 0..list.length() {
        let node = list.item(i).unwrap();
        let elm: web_sys::Element = node.dyn_into().unwrap();

        let inner = elm.inner_html();
        let inner = Arc::new(inner);
        let button = document.create_element("button").unwrap();
        button.set_inner_html("copy");
        let listener = gloo::events::EventListener::new(&button, "click", {
            let inner = inner.clone();
            move |_| {
                let _ = web_sys::window()
                    .unwrap()
                    .navigator()
                    .clipboard()
                    .write_text(&inner);
            }
        });
        listeners.push(listener);
        elm.insert_adjacent_element("beforebegin", &button).unwrap();
        elm.insert_adjacent_element("beforebegin", &document.create_element("p").unwrap())
            .unwrap();

        let Some(class) = elm.get_attribute("class") else {
            continue;
        };
        let Some(ext) = class.strip_prefix("language-") else {
            continue;
        };
        if let Some(rendered) = render_code(ext, &inner) {
            elm.set_inner_html(&rendered);
        }
    }
}

#[component]
pub fn Markdown(content: String) -> Element {
    use pulldown_cmark::Options;
    let options = Options::ENABLE_MATH | Options::ENABLE_TASKLISTS | Options::ENABLE_TABLES;
    let parser = pulldown_cmark::Parser::new_ext(&content, options);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    use_effect(|| {
        spawn(async {});
        render_code_block();
        render_katex();
    });

    rsx! {
        div { class: "markdown", dangerous_inner_html: html }
    }
}
