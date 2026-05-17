use TShare::app::app;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    if let Err(e) = app().await {
        // Always print to stderr so systemd/journalctl shows the failure reason.
        eprintln!("TShare failed to start: {e:#}");
        tracing::error!(target: "system", "Application failed to start: {}", e);
        std::process::exit(1);
    }
}
