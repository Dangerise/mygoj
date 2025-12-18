use super::*;
use std::sync::Mutex;

static RESOURCE: Mutex<String> = Mutex::new(String::new());

pub fn notfound(url: String) {
    *RESOURCE.lock().unwrap() = url;
    navigator().push(Route::NotFound {});
}

#[component]
pub fn NotFound() -> Element {
    let resource=RESOURCE.lock().unwrap().clone();
    rsx! {
        Common { content: "404 not found\nresource {resource}" }
    }
}
