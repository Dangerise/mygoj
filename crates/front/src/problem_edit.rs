use super::*;
use shared::problem::*;
use std::collections::BTreeSet;
use utility::loading_page;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FileState {
    Unchanged,
    Changed,
    Deleted,
    New,
}

#[derive(Debug, PartialEq, Clone)]
enum Event {
    Update,
}

impl FileState {
    fn as_str(&self) -> &'static str {
        use FileState::*;
        match self {
            New => "New",
            Changed => "Changed",
            Deleted => "Deleted",
            Unchanged => "Unchanged",
        }
    }
}

impl std::fmt::Display for FileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct EditingProblemFile {
    file: ProblemFile,
    state: FileState,
}

#[component]
fn render_file_state(state: FileState) -> Element {
    rsx! {
        label { "{state}" }
    }
}

#[component]
fn upload_file(default_path: String) -> Element {
    rsx! {}
}

#[component]
fn render_files_view(
    mut files: Signal<Option<Vec<EditingProblemFile>>>,
    mut events: Signal<Vec<Event>>,
) -> Element {
    let mut at = use_signal(String::new);
    tracing::info!("at {at}");
    let mut show_files = files
        .read()
        .as_ref()
        .unwrap()
        .iter()
        .filter_map(|d| {
            d.file
                .path
                .strip_prefix(&*at.read())
                .filter(|x| !x.contains("/"))
                .map(|x| (x.to_string(), d.clone()))
        })
        .collect::<Vec<_>>();
    show_files.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
    let show_folders = files
        .read()
        .as_ref()
        .unwrap()
        .iter()
        .filter_map(|x| {
            x.file
                .path
                .strip_prefix(&*at.read())
                .filter(|x| x.contains("/"))
                .and_then(|x| x.split("/").next())
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    rsx! {
        if !at.is_empty() {
            p {
                a {
                    onclick: move |_| {
                        let mut at = at.write();
                        at.pop().unwrap();
                        let parent = at.strip_suffix(at.rsplit("/").next().unwrap()).unwrap();
                        *at = parent.into();
                    },
                    ".."
                }
            }
        }
        for folder in show_folders {
            p {
                a {
                    onclick: move |_| {
                        let mut at=at.write();
                        at.push_str(&folder);
                        at.push('/');
                    },
                    "/{folder}"
                }
            }
        }
        for (name , file) in show_files {
            p {
                a { "{name}" }
                render_file_state { state: file.state }
                button { "pub" }
                button { "upd" }
                button { "rm" }
            }
        }
    }
}

#[component]
fn render_files_edit(
    mut files: Signal<Option<Vec<EditingProblemFile>>>,
    mut events: Signal<Vec<Event>>,
) -> Element {
    rsx! {
        render_files_view { files, events }
    }
}

#[component]
fn render_editable(mut editable: Signal<Option<ProblemEditable>>) -> Element {
    let mut time_limit = use_signal(|| editable.read().as_ref().unwrap().time_limit.to_string());
    let mut time_limit_error = use_signal(String::new);
    let mut memory_limit =
        use_signal(|| editable.read().as_ref().unwrap().memory_limit.to_string());
    let mut memory_limit_error = use_signal(String::new);
    use_effect(move || {
        let mut editable = editable.write();
        let editable = editable.as_mut().unwrap();
        let time_limit: u32 = match time_limit.read().parse() {
            Ok(v) => v,
            Err(_) => {
                time_limit_error.set("it should be a non-negative integer".into());
                return;
            }
        };
        if time_limit > 10_000 {
            time_limit_error.set("too large".into());
        }
        editable.time_limit = time_limit;
        let memory_limit: u32 = match memory_limit.read().parse() {
            Ok(v) => v,
            Err(_) => {
                memory_limit_error.set("it should be a non-negative integer".into());
                return;
            }
        };
        if memory_limit > (1 << 24) {
            memory_limit_error.set("too large".into());
            return;
        }
        editable.memory_limit = memory_limit;
    });
    rsx! {
        label { "title" }
        input {
            value: editable.read().as_ref().unwrap().title.clone(),
            onchange: move |evt| {
                editable.write().as_mut().unwrap().title = evt.value();
            },
        }
        label { "time_limit (ms)" }
        input {
            value: time_limit,
            onchange: move |evt| {
                time_limit.set(evt.value());
            },
        }
        {
            let msg = time_limit_error.read();
            if !msg.is_empty() {
                rsx! {
                    label { "{msg}" }
                }
            } else {
                rsx! {}
            }
        }
        label { "memory_limit (mb)" }
        input {
            value: memory_limit,
            onchange: move |evt| {
                memory_limit.set(evt.value());
            },
        }
        {
            let msg = memory_limit_error.read();
            if !msg.is_empty() {
                rsx! {
                    label { "{msg}" }
                }
            } else {
                rsx! {}
            }
        }
        label { "statement" }
        textarea {
            value: editable.read().as_ref().unwrap().statement.clone(),
            onchange: move |evt| {
                editable.write().as_mut().unwrap().title = evt.value();
            },
        }
    }
}

#[component]
pub fn ProblemEdit(pid: Pid) -> Element {
    let mut fetched = use_signal(|| false);
    let mut editable = use_signal(|| None);
    let mut files = use_signal(|| None);
    let events = use_signal(Vec::new);
    {
        let pid = pid.clone();
        use_future(move || {
            let pid = pid.clone();
            async move {
                let editable = async {
                    let editable_val: ProblemEditable =
                        send_message(FrontMessage::GetProblemEditable(pid.clone()))
                            .await
                            .unwrap();
                    editable.set(editable_val.into());
                };
                let files = async {
                    let files_val: Vec<ProblemFile> =
                        send_message(FrontMessage::GetProblemFiles(pid.clone()))
                            .await
                            .unwrap();
                    files.set(Some(
                        files_val
                            .into_iter()
                            .map(|file| EditingProblemFile {
                                file,
                                state: FileState::Unchanged,
                            })
                            .collect::<Vec<_>>(),
                    ));
                };
                futures_util::join!(editable, files);
                fetched.set(true);
            }
        })
    };
    rsx! {
        {
            let pid = pid.0.as_str();
            rsx! {
                h2 { "Edit {pid}" }
                hr {}
            }
        }
        {
            if fetched.cloned() {
                rsx! {
                    render_editable { editable }
                    render_files_edit { files, events }
                    hr {}
                    button { onclick: move |_| {}, "save" }
                }
            } else {
                rsx! {
                    loading_page {}
                }
            }
        }
    }
}
