use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerProgress {
    pub task: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub done: bool,
}

#[derive(Clone)]
pub struct BackgroundWorker {
    app: AppHandle,
    is_busy: Arc<Mutex<bool>>,
}

impl BackgroundWorker {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            is_busy: Arc::new(Mutex::new(false)),
        }
    }

    /// Emits an arbitrary serializable payload to every listening webview.
    pub fn emit_event<T: Serialize + Clone>(&self, event: &str, payload: T) {
        let _ = self.app.emit(event, payload);
    }

    pub async fn emit_progress(
        &self,
        task: &str,
        current: usize,
        total: usize,
        message: &str,
        done: bool,
    ) {
        self.emit_event(
            "worker://progress",
            WorkerProgress {
                task: task.to_string(),
                current,
                total,
                message: message.to_string(),
                done,
            },
        );
    }

    pub async fn run_task<F, Fut>(&self, task_name: &'static str, f: F) -> Result<(), String>
    where
        F: FnOnce(BackgroundWorker) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
    {
        let mut busy = self.is_busy.lock().await;
        if *busy {
            return Err("A background task is already executing".to_string());
        }
        *busy = true;
        drop(busy);

        let worker = self.clone();
        tokio::spawn(async move {
            worker
                .emit_progress(task_name, 0, 100, "Starting...", false)
                .await;
            let res = f(worker.clone()).await;
            if let Err(e) = res {
                worker
                    .emit_progress(task_name, 100, 100, &format!("Failed: {e}"), true)
                    .await;
            } else {
                worker
                    .emit_progress(task_name, 100, 100, "Complete", true)
                    .await;
            }
            let mut busy = worker.is_busy.lock().await;
            *busy = false;
        });

        Ok(())
    }
}
