use super::*;
use itertools::Itertools;
use shared::problem::*;
use std::collections::BTreeSet;
use utility::loading_page;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FileState {
    Unchanged,
    Changed,
    Deleted,
    New,
}

#[derive(Debug, PartialEq, Clone)]
enum EventKind {
    Update(Uuid, Vec<u8>),
    New(Uuid, Vec<u8>),
    ChangeVisibiliy,
    Delete,
}

#[derive(Debug, PartialEq, Clone)]
struct Event {
    on: String,
    kind: EventKind,
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", &self.on, self.kind)
    }
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
fn render_events(events: Signal<Vec<Event>>) -> Element {
    let events = events.read();
    rsx! {
        for evt in &*events {
            p { "{evt}" }
        }
    }
}

#[component]
fn render_file_state(state: FileState) -> Element {
    rsx! {
        label { "{state}" }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UploadedFile {
    path: String,
    uuid: Uuid,
    content: Vec<u8>,
}

#[component]
fn upload_single_file(
    show: Signal<bool>,
    replace_path: String,
    uploaded: Signal<Option<UploadedFile>>,
) -> Element {
    let mut os_path = use_signal(String::new);
    let mut msg = use_signal(|| None);
    rsx! {
        DialogRoot { open: show.cloned(),
            DialogContent {
                DialogTitle { "upload file" }
                DialogDescription { "upload a file to replace {replace_path}" }
                label { "choose a file" }
                p { "{os_path}" }
                input {
                    id: "file-input",
                    r#type: "file",
                    onchange: move |evt| {
                        let files = evt.files();
                        if files.len() == 1 {
                            os_path.set(files[0].name());
                            msg.set(None);
                        } else {
                            os_path.set(String::new());
                            msg.set(Some("you can only select one file"));
                        }
                    },
                }
                if let Some(msg) = &*msg.read() {
                    label { "{msg}" }
                }
                button { onclick: move |_| {}, "confirm" }
            }
        }
    }
}

#[component]
fn render_files_view(
    mut files: Signal<Option<Vec<EditingProblemFile>>>,
    mut events: Signal<Vec<Event>>,
) -> Element {
    let mut show_upload = use_signal(|| false);
    let mut replace_path = use_signal(String::new);
    let mut uploaded = use_signal(|| None);

    let mut change_visibility = move |file: &str| {
        let mut events = events.write();
        for idx in 0..events.len() {
            let evt = &events[idx];
            if evt.on == file && matches!(evt.kind, EventKind::ChangeVisibiliy) {
                events.remove(idx);
                return;
            }
        }
        events.push(Event {
            on: file.into(),
            kind: EventKind::ChangeVisibiliy,
        });
    };
    let is_visibility_changed = |file: &str| {
        events
            .read()
            .iter()
            .any(|x| x.on == file && x.kind == EventKind::ChangeVisibiliy)
    };
    let is_content_changed = |file: &str| {
        events
            .read()
            .iter()
            .any(|x| x.on == file && matches!(x.kind, EventKind::Update(_, _)))
    };
    let is_removed = move |file: &str| {
        events
            .read()
            .iter()
            .any(|x| x.on == file && matches!(x.kind, EventKind::Delete))
    };
    let mut remove = move |file: &str| {
        assert!(!is_removed(file));
        events.write().push(Event {
            on: file.into(),
            kind: EventKind::Delete,
        });
    };
    let mut recover = move |file: &str| {
        assert!(is_removed(file));
        let mut events = events.write();
        let idx = events
            .iter()
            .enumerate()
            .filter_map(|(i, x)| matches!(x.kind, EventKind::Delete).then_some(i))
            .next()
            .unwrap();
        events.remove(idx);
    };

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
        .update(|(_, d)| {
            if is_removed(&d.file.path) {
                d.state = FileState::Deleted;
            } else {
                if matches!(d.state, FileState::Unchanged | FileState::Changed) {
                    d.state =
                        if is_visibility_changed(&d.file.path) || is_content_changed(&d.file.path) {
                            FileState::Changed
                        } else {
                            FileState::Unchanged
                        }
                }
            }
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
        upload_single_file { show: show_upload, replace_path, uploaded }
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
                        let mut at = at.write();
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
                {
                    let visibiliy = file.file.is_public ^ is_visibility_changed(&file.file.path);
                    let text = if visibiliy { "pub" } else { "priv" };
                    let path = file.file.path.clone();
                    if is_removed(&path) {
                        rsx! {
                            button {
                                onclick: move |_| {
                                    recover(&path);
                                },
                                "bk"
                            }
                        }
                    } else {
                        let p2 = path.clone();
                        let p3 = path.clone();
                        rsx! {
                            button {
                                onclick: move |_| {
                                    change_visibility(&path);
                                },
                                "{text} "
                            }
                            button {
                                onclick: move |_| {
                                    show_upload.set(true);
                                    replace_path.set(p3.to_string());
                                    uploaded.set(None);
                                },
                                "upd "
                            }
                            button {
                                onclick: {
                                    move |_| {
                                        remove(&p2);
                                    }
                                },
                                "rm"
                            }
                        }
                    }
                }
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
                    render_events { events }
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
