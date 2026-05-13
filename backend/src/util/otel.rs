use anyhow::Context as _;
use opentelemetry::global;
use opentelemetry_otlp::{MetricExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;

pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl OtelGuard {
    pub fn shutdown(self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            tracing::warn!(error = %err, "otel tracer shutdown failed");
        }
        if let Err(err) = self.meter_provider.shutdown() {
            tracing::warn!(error = %err, "otel meter shutdown failed");
        }
    }
}

pub fn init(service_name: &'static str) -> anyhow::Result<OtelGuard> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let resource = Resource::builder().with_service_name(service_name).build();

    let span_exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .context("failed to build OTLP span exporter")?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(resource.clone())
        .build();
    global::set_tracer_provider(tracer_provider.clone());

    let metric_exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .context("failed to build OTLP metric exporter")?;

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(resource)
        .build();
    global::set_meter_provider(meter_provider.clone());

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
    })
}

pub fn tracer() -> opentelemetry::global::BoxedTracer {
    global::tracer("kioku-backend")
}
