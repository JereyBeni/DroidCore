//! DroidCore desktop application entry point.
//!
//! Stage 1 of the roadmap: this binary only initializes logging and
//! reports that the workspace builds. The egui interface arrives in stage 2.

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "DroidCore starting (workspace skeleton, no emulation yet)"
    );
}
