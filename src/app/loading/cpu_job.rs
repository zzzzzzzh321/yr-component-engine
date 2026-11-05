//! Owned CPU work, polled only after returning to the window event loop.
//! Dropping a job discards its unpublished result without joining its worker.

use std::{sync::mpsc, task::Poll};

pub(crate) struct CpuLoadJob<T> {
    result: mpsc::Receiver<T>,
}

impl<T: Send + 'static> CpuLoadJob<T> {
    pub(crate) fn start(
        name: &str,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> anyhow::Result<Self> {
        let (sender, result) = mpsc::channel();
        let label = name.to_owned();
        let work = move || {
            let started = crate::util::time::Instant::now();
            let output = work();
            log::info!(target: "vera20k::loading_attempt", "cpu_complete {} elapsed_ms={}", label, started.elapsed().as_millis());
            output
        };
        #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
        std::thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                let _ = sender.send(work());
            })?;
        #[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
        {
            let _ = name;
            let _ = sender.send(work());
        }
        Ok(Self { result })
    }

    pub(crate) fn poll(&mut self) -> anyhow::Result<Poll<T>> {
        match self.result.try_recv() {
            Ok(result) => Ok(Poll::Ready(result)),
            Err(mpsc::TryRecvError::Empty) => Ok(Poll::Pending),
            Err(mpsc::TryRecvError::Disconnected) => {
                anyhow::bail!("loading CPU worker disconnected")
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn unfinished_work_yields_and_preserves_its_error() {
        let (release, gate) = mpsc::channel();
        let mut job = CpuLoadJob::start("loading-test", move || {
            gate.recv().unwrap();
            Err::<(), _>("original failure")
        })
        .unwrap();
        assert!(matches!(job.poll().unwrap(), Poll::Pending));
        release.send(()).unwrap();
        assert_eq!(job.result.recv().unwrap(), Err("original failure"));
    }

    #[test]
    fn cancelled_work_does_not_block_or_publish_its_result() {
        struct Output(mpsc::Sender<()>);
        impl Drop for Output {
            fn drop(&mut self) {
                self.0.send(()).unwrap();
            }
        }
        let (release, gate) = mpsc::channel();
        let (discarded, observe_discard) = mpsc::channel();
        let job = CpuLoadJob::start("loading-cancel-test", move || {
            gate.recv().unwrap();
            Output(discarded)
        })
        .unwrap();
        drop(job);
        release.send(()).unwrap();
        observe_discard.recv().unwrap();
    }

    #[test]
    fn disconnected_worker_is_an_error_instead_of_permanent_pending() {
        let (sender, result) = mpsc::channel::<()>();
        drop(sender);
        let mut job = CpuLoadJob { result };
        assert!(job.poll().unwrap_err().to_string().contains("disconnected"));
    }
}
