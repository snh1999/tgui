use crate::process::log_buffer::LogBuffer;
use crate::process::managed_process::ProcessEvent;
use crate::process::models::{LogLineEvent, StreamingConfig};
use crate::utils::get_utc_timestamp_string;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};

/// Handles streaming stdout/stderr from a child process
pub struct LogStreamer {
    execution_id: i64,
    pid: u32,
    buffer: Arc<RwLock<LogBuffer>>,
    config: StreamingConfig,
}

impl LogStreamer {
    pub fn new(
        execution_id: i64,
        pid: u32,
        config: StreamingConfig,
    ) -> (Self, Arc<RwLock<LogBuffer>>) {
        let buffer = Arc::new(RwLock::new(LogBuffer::new(config.buffer_capacity)));

        let streamer = Self {
            execution_id,
            pid,
            buffer: buffer.clone(),
            config,
        };

        (streamer, buffer)
    }

    pub fn start(
        self,
        stdout: Option<ChildStdout>,
        stderr: Option<ChildStderr>,
        event_sender: mpsc::Sender<ProcessEvent>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut stdout_lines = stdout.map(|s| BufReader::new(s).lines());
            let mut stderr_lines = stderr.map(|s| BufReader::new(s).lines());

            let mut batch: Vec<Arc<LogLineEvent>> = Vec::with_capacity(self.config.batch_size);
            let timeout_duration = tokio::time::Duration::from_millis(self.config.batch_timeout_ms);
            let mut next_emit = tokio::time::Instant::now() + timeout_duration;

            loop {
                tokio::select! {
                    result = Self::read_from(&mut stdout_lines), if stdout_lines.is_some() => {
                        match result {
                            Some(Ok(line)) => {
                                let log_line = self.create_log_line(line, false);
                                batch.push(log_line);

                                if batch.len() >= self.config.batch_size {
                                    next_emit = self.emit_batch(&event_sender, &mut batch, timeout_duration).await;
                                }
                            }
                           Some(Err(e)) => {
                                warn!(error = %e, "Error reading stdout");
                            }
                            None => {
                                stdout_lines = None;
                            }
                        }
                    }

                    result = Self::read_from(&mut stderr_lines), if stderr_lines.is_some() => {
                        match result {
                            Some(Ok(line)) => {
                                let log_line = self.create_log_line(line, true);
                                batch.push(log_line);

                                if batch.len() >= self.config.batch_size {
                                    next_emit = self.emit_batch(&event_sender, &mut batch, timeout_duration).await;
                                }
                            }
                            Some(Err(e)) => {
                                warn!(error = %e, "Error reading stderr");
                            }
                            None => {
                                stderr_lines = None;
                            }
                        }
                    }

                    // Emit batch if timeout/size reached
                    _ = tokio::time::sleep_until(next_emit), if !batch.is_empty() => {
                        next_emit = self.emit_batch(&event_sender, &mut batch, timeout_duration).await;
                    }

                    else => break,
                }
                // streams closed
                if stdout_lines.is_none() && stderr_lines.is_none() {
                    if !batch.is_empty() {
                        let _ = self
                            .emit_batch(&event_sender, &mut batch, timeout_duration)
                            .await;
                    }
                    break;
                }
            }

            debug!(
                execution_id = self.execution_id,
                pid = self.pid,
                "Log streaming ended"
            );
        })
    }

    async fn read_from<T>(
        reader: &mut Option<tokio::io::Lines<BufReader<T>>>,
    ) -> Option<std::io::Result<String>>
    where
        T: AsyncRead + Unpin,
    {
        match reader {
            Some(lines) => lines.next_line().await.transpose(),
            None => None,
        }
    }

    fn create_log_line(&self, mut content: String, is_stderr: bool) -> Arc<LogLineEvent> {
        if content.len() > self.config.max_line_length {
            let mut truncate_at = self.config.max_line_length;
            while !content.is_char_boundary(truncate_at) && truncate_at > 0 {
                truncate_at -= 1;
            }
            content.truncate(truncate_at);
            content.push_str("...[truncated]");
        }

        Arc::new(LogLineEvent {
            timestamp: get_utc_timestamp_string(),
            content,
            is_stderr,
            execution_id: self.execution_id,
        })
    }

    /// Writes batch to the log buffer and forwards it to the event sender.
    async fn emit_batch(
        &self,
        event_sender: &mpsc::Sender<ProcessEvent>,
        batch: &mut Vec<Arc<LogLineEvent>>,
        timeout_duration: tokio::time::Duration,
    ) -> tokio::time::Instant {
        let batch_to_emit = std::mem::take(batch);
        self.buffer.write().await.push_many(batch_to_emit.clone());

        let event = ProcessEvent::LogBatch(batch_to_emit);

        if let Err(_) = event_sender.try_send(event) {
            warn!("Dropping log batch");
        }
        tokio::time::Instant::now() + timeout_duration
    }

    pub fn spawn_log_streaming(
        execution_id: i64,
        pid: u32,
        stdout: Option<ChildStdout>,
        stderr: Option<ChildStderr>,
        config: StreamingConfig,
        event_sender: mpsc::Sender<ProcessEvent>,
    ) -> (Arc<RwLock<LogBuffer>>, tokio::task::JoinHandle<()>) {
        let (streamer, buffer) = LogStreamer::new(execution_id, pid, config);
        let handle = streamer.start(stdout, stderr, event_sender);
        (buffer, handle)
    }
}
