use rocket::futures::future::BoxFuture;
use rocket::{Orbit, Rocket};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

pub fn init_tracer() -> () {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("golinks-rs")
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Failed to start tracer");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(telemetry)
        .init();
}

pub fn shutdown_tracer<'a>(_: &'a Rocket<Orbit>) -> BoxFuture<'a, ()> {
    Box::pin(async move {
        opentelemetry::global::shutdown_tracer_provider();
    })
}
