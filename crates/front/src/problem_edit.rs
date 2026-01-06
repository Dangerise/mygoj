use super::*;
use compact_str::CompactString;
use itertools::Itertools;
use shared::problem::*;
use std::collections::{BTreeMap, BTreeSet};
use utility::loading_page;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
struct UploadedFile {
    path: CompactString,
    uuid: Uuid,
    content: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FileState {
    Unchanged,
    Changed,
    Removed,
    New,
}

#[derive(Debug, PartialEq, Clone)]
enum EventKind {
    Update(UploadedFile),
    New(UploadedFile),
    SetPub(CompactString),
    SetPriv(CompactString),
    Remove(CompactString),
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
            Removed => "Deleted",
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
    is_selected: bool,
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
    tracing::info!("{files:#?}");

    fn walk(
        mut files: Signal<Option<Vec<EditingProblemFile>>>,
        folder: &str,
        f: impl Fn(&mut EditingProblemFile),
    ) {
        files
            .write()
            .as_mut()
            .unwrap()
            .iter_mut()
            .filter(|d| d.file.path.strip_prefix(folder).is_some())
            .for_each(f);
    }

    let mut show_upload = use_signal(|| false);
    let mut replace_path = use_signal(String::new);
    let mut uploaded = use_signal(|| None);

    let mut at = use_signal(String::new);

    let folder_path = {
        move |folder: &str| {
            if folder.is_empty() {
                format!("{at}")
            } else {
                format!("{at}{folder}/")
            }
        }
    };

    let file_directly_in_folder = |file: &str, folder: &str| -> Option<CompactString> {
        file.strip_prefix(&folder_path(folder))
            .filter(|s| !s.contains("/"))
            .map(CompactString::from)
    };

    let file_in_folder =
        move |file: &str, folder: &str| -> bool { file.strip_prefix(&folder_path(folder)).is_some() };

    let mut select_folder = move |folder: &str, is_selected: bool| {
        files
            .write()
            .as_mut()
            .unwrap()
            .iter_mut()
            .filter(|d| file_in_folder(&d.file.path, folder))
            .for_each(|d| d.is_selected = is_selected);
    };

    let mut select_file = move |file: &str, is_selected| {
        files
            .write()
            .as_mut()
            .unwrap()
            .iter_mut()
            .filter(|d| d.file.path == file)
            .for_each(|d| d.is_selected = is_selected);
    };

    tracing::info!("at {at}");
    let mut show_files = files
        .read()
        .as_ref()
        .unwrap()
        .iter()
        .filter_map(|d| file_directly_in_folder(&d.file.path, "").map(|x| (x, d.clone())))
        .collect::<Vec<_>>();
    show_files.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    #[derive(Debug)]
    struct Folder {
        is_selected: bool,
    }

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
                .map(CompactString::from)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|s| {
            let meta = Folder {
                is_selected: files
                    .read()
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|d| file_in_folder(&d.file.path, &s))
                    .update(|d| tracing::info!("{} in {}", &d.file.path, folder_path(&s)))
                    .all(|d| d.is_selected),
            };
            (s, meta)
        })
        .collect::<BTreeMap<_, _>>();

    tracing::info!("folders {show_folders:#?}");

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
        for (folder , meta) in show_folders {
            p {
                input {
                    r#type: "checkbox",
                    checked: meta.is_selected,
                    onchange: {
                        {
                            let folder=folder.clone();
                            move |evt| {
                                let folder = folder.clone();
                                select_folder(&folder,evt.checked());
                            }
                        }
                    },
                }
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
                input {
                    r#type: "checkbox",
                    checked: file.is_selected,
                    onchange: {
                        move |evt| {
                            let path = file.file.path.clone();
                            select_file(&path, evt.checked());
                        }
                    },
                }
                a { "{name}" }
                render_file_state { state: file.state }
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
        hr {}
        render_events { events }
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
                                is_selected: false,
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
                    hr {}
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
