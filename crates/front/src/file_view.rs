use super::*;

#[component]
fn Text(content: String) -> Element {
    tracing::info!("view file as plain text");
    rsx! {
        for line in content.lines() {
            p { "{line}" }
        }
    }
}

#[component]
fn InvalideUtf8() -> Element {
    rsx! { "Invalid utf-8" }
}

#[component]
fn Unsupported() -> Element {
    rsx! { "this file is not supported to be view" }
}

#[component]
pub fn FileView(name: String, bytes: bytes::Bytes) -> Element {
    let ext = name
        .rsplit_once(".")
        .map(|(_, s)| s)
        .unwrap_or(name.as_str());

    if bytes.len() > (1 << 20) {
        return rsx! { "too large to view" };
    }

    let as_string = || {
        let bytes = &*bytes;
        str::from_utf8(bytes).map(|s| s.to_string())
    };
    match ext {
        "md" => {
            tracing::info!("view file as markdown");
            rsx! {
                if let Ok(s) = as_string() {
                    Markdown { content: s }
                } else {
                    InvalideUtf8 {}
                }
            }
        }
        "txt"=>{
            rsx!{
                if let Ok(s) = as_string() {
                    Text { content: s }
                } else {
                    InvalideUtf8 {}
                }
            }
        }
        _ => rsx!{
            if let Ok(s) = as_string() {
                Text { content: s }
            } else {
                Unsupported {}
            }
        },
    }
}