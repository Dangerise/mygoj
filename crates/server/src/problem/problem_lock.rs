use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq)]
enum RequestKind {
    Read,
    Write,
}

#[derive(Debug)]
struct Request {
    sender: oneshot::Sender<()>,
    kind: RequestKind,
}

#[derive(Debug)]
struct Inner {
    count: u64,
    queue: VecDeque<Request>,
    kind: RequestKind,
}

#[derive(Debug)]
pub struct ProblemLock {
    inner: Mutex<Inner>,
}

impl Default for ProblemLock {
    fn default() -> Self {
        Self::new()
    }
}

impl ProblemLock {
    fn next(&self) {
        let mut lock = self.inner.lock().unwrap();
        let Inner { count, queue, kind } = &mut *lock;
        if let Some(front) = queue.pop_front() {
            assert_eq!(*count, 0);
            *kind = front.kind;
            match front.kind {
                RequestKind::Read => {
                    let mut sender = front.sender;
                    loop {
                        *count += 1;
                        sender.send(()).unwrap();
                        if matches!(
                            queue.front(),
                            Some(Request {
                                kind: RequestKind::Read,
                                ..
                            })
                        ) {
                            sender = queue.pop_front().unwrap().sender;
                        } else {
                            break;
                        }
                    }
                }
                RequestKind::Write => {
                    *count += 1;
                    front.sender.send(()).unwrap();
                }
            }
        }
    }

    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                count: 0,
                queue: VecDeque::new(),
                kind: RequestKind::Read,
            }),
        }
    }

    pub async fn read_lock(&self) {
        let receiver;
        {
            let mut lock = self.inner.lock().unwrap();
            let Inner { count, queue, kind } = &mut *lock;
            if *count == 0 {
                *count = 1;
                *kind = RequestKind::Read;
                return;
            }
            if *kind == RequestKind::Read {
                *count += 1;
                receiver = None;
            } else {
                let (sender, receiver2) = oneshot::channel();
                receiver = Some(receiver2);
                queue.push_back(Request {
                    sender,
                    kind: RequestKind::Read,
                });
            }
        }
        if let Some(recv) = receiver {
            recv.await.unwrap();
        }
    }

    #[track_caller]
    pub fn read_unlock(&self) {
        let mut lock = self.inner.lock().unwrap();
        let Inner {
            count,
            queue: _,
            kind,
        } = &mut *lock;
        assert_eq!(*kind, RequestKind::Read);
        assert!(*count > 0);
        *count -= 1;
        if *count == 0 {
            drop(lock);
            self.next();
        }
    }

    pub async fn write_lock(&self) {
        let receiver;
        {
            let mut lock = self.inner.lock().unwrap();
            let Inner { count, queue, kind } = &mut *lock;
            if *count == 0 {
                *kind = RequestKind::Write;
                *count = 1;
                return;
            } else {
                let sender;
                (sender, receiver) = oneshot::channel();
                queue.push_back(Request {
                    sender,
                    kind: RequestKind::Write,
                });
            }
        }
        receiver.await.unwrap()
    }

    #[track_caller]
    pub fn write_unlock(&self) {
        let mut lock = self.inner.lock().unwrap();
        let Inner {
            count,
            queue: _,
            kind,
        } = &mut *lock;
        assert_eq!(*kind, RequestKind::Write);
        assert_eq!(*count, 1);
        *count = 0;
        drop(lock);
        self.next();
    }
}

#[test]
fn basic() {
    tracing_subscriber::fmt().init();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    use static_init::dynamic;
    use std::time::Duration;

    #[dynamic]
    static LOCK: ProblemLock = ProblemLock::new();
    #[dynamic]
    static S: Mutex<Vec<String>> = Mutex::new(Vec::new());

    rt.block_on(async {
        for i in 0..10 {
            tokio::spawn(async move {
                tracing::info!("begin lock {i}");
                LOCK.read_lock().await;
                tracing::info!("end lock {i}");

                use rand::Rng;
                let time = rand::rng().random_range(700..=1000);
                tokio::time::sleep(Duration::from_millis(time)).await;

                S.lock().unwrap().push(format!("read"));

                tracing::info!("begin unlock {i}");
                LOCK.read_unlock();
                tracing::info!("end unlock {i}");
            });
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!("begin write lock");
        LOCK.write_lock().await;
        tracing::info!("end write lock");

        S.lock().unwrap().push("write".to_string());

        tracing::info!("begin write unlock");
        LOCK.write_unlock();
        tracing::info!("end write unlock");

        tracing::info!("begin extra lock");
        LOCK.read_lock().await;
        tracing::info!("end extra lock");

        S.lock().unwrap().push("extra_read".to_string());

        tracing::info!("begin extra unlock");
        LOCK.read_unlock();
        tracing::info!("end extra unlock");
    });
    let mut exp = vec!["read".to_string(); 10];
    exp.append(&mut vec!["write".into(), "extra_read".into()]);
    assert_eq!(&*S.lock().unwrap(), &exp);
}
