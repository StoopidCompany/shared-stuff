fn main() {
    stoopid_logging::init().expect("install subscriber");

    tracing::info!("service started");
    tracing::warn!(downstream = "billing", elapsed_ms = 842, "downstream slow");

    let span = tracing::info_span!("request", request_id = "req_42");
    let _enter = span.enter();
    tracing::info!(extra = 1, "inside span");
}
