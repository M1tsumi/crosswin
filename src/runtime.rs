#[cfg(not(feature = "tokio"))]
compile_error!("crosswin MVP currently requires the `tokio` feature to be enabled.");

/// Spawn a future onto the Tokio runtime.
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future)
}

/// Run a future to completion on a new single-threaded Tokio runtime.
///
/// This is a convenience wrapper for calling async crosswin APIs from
/// synchronous contexts without managing a `tokio::runtime::Runtime` manually.
///
/// # Example
/// ```rust,no_run
/// use crosswin::runtime::block_on;
/// use crosswin::processes::list_processes;
///
/// let processes = block_on(list_processes()).unwrap();
/// println!("found {} processes", processes.len());
/// ```
pub fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("failed to create Tokio runtime")
        .block_on(f)
}
