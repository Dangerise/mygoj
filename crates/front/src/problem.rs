use super::*;
pub use shared::problem::ProblemFront;

async fn get_problem_front(pid: &str) -> eyre::Result<ProblemFront> {
    let url = format!("{}/api/problem_front?pid={}", *SERVER_ORIGIN, pid);
    tracing::info!("get problem front from {url}");
    let res = reqwest::get(url).await?.json().await?;
    Ok(res)
}

#[component]
fn loading_page() -> Element {
    rsx! {
        p { "Loading, please wait`" }
    }
}

#[component]
fn wrong_page() -> Element {
    rsx! {
        p {  "Something goes wrong "}
    }
}

#[component]
fn render_problem(front: ProblemFront) -> Element {
    tracing::info!("render problem {:?}", &front);
    let ProblemFront {
        pid,
        title,
        statement,
    } = front;
    rsx! {
        h1 { "{pid} {title}" }
        p { "{statement}" }
    }
}

#[component]
pub fn Problem(pid: String) -> Element {
    let front = {
        let pid = pid.clone();
        use_resource(move || {
            let pid = pid.clone();
            async move { get_problem_front(&pid).await }
        })
    };
    if let Some(front) = &*front.read() {
        match front {
            Ok(front) => rsx! {
                render_problem { front:front.clone() }
            },
            Err(err) => {
                tracing::error!("error {:?}", err);
                rsx! {
                    wrong_page {  }
                }
            }
        }
    } else {
        rsx! {
            loading_page{}
        }
    }
}
