use super::*;
use bytes::Bytes;
use compact_str::CompactString;
use dioxus::html::FileData;
use shared::problem::*;
use utility::loading_page;

#[derive(Clone, PartialEq)]
struct UploadedFile {
    path: CompactString,
    content: Bytes,
}

impl std::fmt::Debug for UploadedFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.path)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FileState {
    Unchanged,
    Changed,
    Removed,
    New,
}

#[derive(Debug, PartialEq, Clone)]
enum Event {
    Update(UploadedFile),
    New(UploadedFile),
    SetPub(CompactString),
    SetPriv(CompactString),
    Remove(CompactString),
}

#[derive(Debug, PartialEq, Clone)]
struct EventGroup {
    evts: Vec<Event>,
}

impl FileState {
    fn as_str(&self) -> &'static str {
        use FileState::*;
        match self {
            New => "New",
            Changed => "Changed",
            Removed => "Removed",
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
    is_public: bool,
    path: CompactString,
    size: u64,
    last_modified: i64,
    state: FileState,
    is_selected: bool,
}

#[component]
fn render_events(evt_groups: Signal<Vec<EventGroup>>) -> Element {
    let groups = evt_groups.read();
    rsx! {
        for g in &*groups {
            for e in &g.evts {
                {format!("{e:#?}")}
            }
            hr {}
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
fn upload_files(show: Signal<bool>, uploaded: Signal<Vec<UploadedFile>>) -> Element {
    let mut selected: Signal<Vec<FileData>> = use_signal(Vec::new);
    let mut err = use_signal(String::new);
    rsx! {
        DialogRoot { open: show.cloned(),
            DialogContent {
                DialogTitle { "upload file" }
                DialogDescription { "upload files" }
                label { "choose some files" }
                input {
                    id: "file-input",
                    r#type: "file",
                    multiple: true,
                    onchange: move |evt| {
                        let files = evt.files();
                        tracing::info!("get files {files:#?}");
                        selected.set(files);
                    },
                }
                {selected.iter().map(|f| f.name()).map(|f| rsx! {
                    p { "{f}" }
                })}
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let selected = selected.read();
                            let files = selected.iter().map(|f| f.read_bytes());
                            let files = futures_util::future::join_all(files).await;
                            let mut uploaded = uploaded.write();
                            for (meta, content) in selected.iter().zip(files) {
                                let content = match content {
                                    Ok(v) => v,
                                    Err(e) => {
                                        err.set(format!("{e:#?}"));
                                        return;
                                    }
                                };
                                uploaded
                                    .push(UploadedFile {
                                        path: meta.name().into(),
                                        content,
                                    });
                            }
                            show.set(false);
                        });
                    },
                    "confirm"
                }
            }
        }
    }
}

#[component]
fn render_files_view(
    mut files: Signal<Option<Vec<EditingProblemFile>>>,
    mut evt_groups: Signal<Vec<EventGroup>>,
) -> Element {
    let mut add_group = move |evts: Vec<_>| {
        evt_groups.push(EventGroup { evts });
    };

    tracing::info!("{files:#?}");

    let mut show_upload = use_signal(|| false);
    let mut uploaded = use_signal(Vec::<UploadedFile>::new);

    if !show_upload.cloned() && !uploaded.is_empty() {
        let uploaded = uploaded.replace(Vec::new());
        let mut files = files.as_mut().unwrap();
        let mut group = Vec::with_capacity(uploaded.len());
        for new_file in uploaded {
            let time = web_sys::js_sys::Date::now() / 1000.;
            let size = new_file.content.len() as u64;
            let last_modified = time as i64;
            let path = new_file.path.clone();
            let Some(old) = files.iter_mut().filter(|d| d.path == path).next() else {
                files.push(EditingProblemFile {
                    is_public: false,
                    size,
                    last_modified,
                    path,
                    state: FileState::New,
                    is_selected: false,
                });
                group.push(Event::New(new_file));
                continue;
            };
            *old = EditingProblemFile {
                path,
                size,
                last_modified,
                state: FileState::Changed,
                ..*old
            };
            group.push(Event::Update(new_file));
        }
        add_group(group);
    }

    if !files
        .as_ref()
        .unwrap()
        .iter()
        .is_sorted_by(|a, b| a.path < b.path)
    {
        files
            .as_mut()
            .unwrap()
            .sort_unstable_by(|a, b| a.path.cmp(&b.path));
    }

    let mut shift_button = use_signal(|| false);
    let mut last_selection = use_signal(|| None);

    rsx! {
        div {
            tabindex: 0,
            onkeydown: move |evt| {
                if evt.code() == Code::ShiftLeft {
                    evt.prevent_default();
                    shift_button.set(true);
                    tracing::info!("shift on");
                }
            },
            onkeyup: move |evt| {
                if evt.code() == Code::ShiftLeft {
                    shift_button.set(false);
                    tracing::info!("shift down");
                }
            },
            button {
                onclick: move |_| {
                    files.as_mut().unwrap().iter_mut().for_each(|x| x.is_selected = false);
                },
                "clear selection"
            }
            upload_files { show: show_upload, uploaded }
            for (idx , file) in files.as_ref().unwrap().iter().enumerate() {
                {
                    rsx! {
                        p {
                            onclick: {
                                move |_| {
                                    if let Some(last) = last_selection.cloned() && shift_button.cloned() {
                                        let (left, right) = if last < idx { (last, idx) } else { (idx, last) };
                                        let mut files = files.as_mut().unwrap();
                                        for idx in left..=right {
                                            files[idx].is_selected = true;
                                        }
                                    } else {
                                        files.as_mut().unwrap()[idx].is_selected ^= true;
                                    }
                                    last_selection.set(Some(idx));
                                }
                            },
                            input { r#type: "checkbox", checked: file.is_selected }
                            a { {file.path.as_str()} }
                            {"    "}
                            {if file.is_public { "pub" } else { "priv" }}
                            {"    "}
                            render_file_state { state: file.state }
                        }
                    }
                }
            }
            button {
                onclick: move |_| {
                    add_group(
                        files
                            .as_mut()
                            .unwrap()
                            .iter_mut()
                            .filter(|d| d.is_selected && d.state != FileState::Removed)
                            .map(|d| {
                                d.state = FileState::Removed;
                                Event::Remove(d.path.clone())
                            })
                            .collect(),
                    );
                },
                "remove"
            }
            button {
                onclick: move |_| {
                    add_group(
                        files
                            .as_mut()
                            .unwrap()
                            .iter_mut()
                            .filter(|d| {
                                d.is_selected && d.state != FileState::Removed && !d.is_public
                            })
                            .map(|d| {
                                d.state = FileState::Changed;
                                d.is_public = true;
                                Event::SetPub(d.path.clone())
                            })
                            .collect(),
                    );
                },
                "set pub"
            }
            button {
                onclick: move |_| {
                    add_group(
                        files
                            .as_mut()
                            .unwrap()
                            .iter_mut()
                            .filter(|d| {
                                d.is_selected && d.state != FileState::Removed && d.is_public
                            })
                            .map(|d| {
                                d.state = FileState::Changed;
                                d.is_public = false;
                                Event::SetPriv(d.path.clone())
                            })
                            .collect(),
                    );
                },
                "set priv"
            }
            button {
                onclick: move |_| {
                    show_upload.set(true);
                },
                "upload"
            }
        }

    }
}

#[component]
fn render_files_edit(
    mut files: Signal<Option<Vec<EditingProblemFile>>>,
    mut evt_groups: Signal<Vec<EventGroup>>,
) -> Element {
    rsx! {
        render_files_view { files, evt_groups }
        hr {}
        render_events { evt_groups }
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
    let evt_groups = use_signal(Vec::new);
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
                            .map(
                                |ProblemFile {
                                     path,
                                     uuid: _,
                                     is_public,
                                     size,
                                     last_modified,
                                 }| EditingProblemFile {
                                    is_public,
                                    path,
                                    size,
                                    last_modified,
                                    state: FileState::Unchanged,
                                    is_selected: false,
                                },
                            )
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
                    render_files_edit { files, evt_groups }
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
