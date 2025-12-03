#[cfg(not(feature = "tokio"))]
compile_error!("crosswin MVP currently requires the `tokio` feature to be enabled.");

pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future)
}
