// AUTOGENERTED Components module
pub mod dialog;
pub use dialog::*;

use dioxus::prelude::*;

#[component]
pub fn Multilines(content: String) -> Element {
    let lines = content.lines();
    rsx! {
        for line in lines {
            p { "{line}" }
        }
    }
}
