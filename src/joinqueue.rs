use std::collections::VecDeque;
use std::pin::Pin;
use tokio::task::JoinSet;

#[derive(Debug)]
pub struct JoinQueue<T> {
    max_concurrent: usize,
    queue: VecDeque<QueueTask<T>>,
    handle: JoinSet<T>,
}

enum QueueTask<T> {
    Blocking(Box<dyn FnOnce() -> T + Send + 'static>),
    NonBlocking(Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>),
}

impl<T: Send + 'static> JoinQueue<T> {
    #[must_use]
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            queue: VecDeque::new(),
            handle: JoinSet::new(),
        }
    }
    fn spawn_max_from_queue(&mut self) {
        while self.handle.len() < self.max_concurrent {
            let Some(next_task) = self.queue.pop_back() else {
                break;
            };
            match next_task {
                QueueTask::Blocking(task) => {
                    self.handle.spawn_blocking(task);
                }
                QueueTask::NonBlocking(task) => {
                    self.handle.spawn(task);
                }
            }
        }
    }
    pub fn enqueue<F: std::future::Future<Output = T> + Send + 'static>(
        &mut self,
        task: F,
    ) -> bool {
        if self.handle.len() < self.max_concurrent {
            self.handle.spawn(task);
            true
        } else {
            self.queue.push_back(QueueTask::NonBlocking(Box::pin(task)));
            false
        }
    }
    pub fn enqueue_blocking<F: FnOnce() -> T + Send + 'static>(&mut self, task: F) -> bool {
        if self.handle.len() < self.max_concurrent {
            self.handle.spawn_blocking(task);
            true
        } else {
            self.queue.push_back(QueueTask::Blocking(Box::new(task)));
            false
        }
    }

    #[must_use]
    pub async fn join_next(&mut self) -> Option<Result<T, tokio::task::JoinError>> {
        if let Some(join_result) = self.handle.join_next().await {
            self.spawn_max_from_queue();
            Some(join_result)
        } else if let Some(next_task) = self.queue.pop_front() {
            let join_result = match next_task {
                QueueTask::Blocking(task) => match tokio::task::spawn_blocking(task).await {
                    Ok(it) => it,
                    Err(err) => return Some(Err(err)),
                },
                QueueTask::NonBlocking(task) => task.await,
            };
            self.spawn_max_from_queue();
            Some(Ok(join_result))
        } else {
            None
        }
    }
}

impl<T> std::fmt::Debug for QueueTask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocking(_) => f.debug_tuple("Blocking").finish(),
            Self::NonBlocking(_) => f.debug_tuple("NonBlocking").finish(),
        }
    }
}
