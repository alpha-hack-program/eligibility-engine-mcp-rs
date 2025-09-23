use once_cell::sync::Lazy;
use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts, Registry};

pub static METRICS: Lazy<EligibilityMetrics> = Lazy::new(|| EligibilityMetrics::new());

pub struct EligibilityMetrics {
    pub registry: Registry,
    pub requests_total: Counter,
    pub errors_total: Counter,
    pub request_duration: Histogram,
    pub active_requests: Gauge,
}

impl EligibilityMetrics {
    fn new() -> Self {
        let registry = Registry::new();

        let requests_total = Counter::with_opts(
            Opts::new(
                "eligibility_requests_total",
                "Total number of unpaid leave eligibility evaluation requests"
            )
        ).unwrap();

        let errors_total = Counter::with_opts(
            Opts::new(
                "eligibility_errors_total",
                "Total number of errors in unpaid leave eligibility evaluations"
            )
        ).unwrap();

        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "eligibility_request_duration_seconds",
                "Duration of unpaid leave eligibility evaluation requests in seconds"
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0])
        ).unwrap();

        let active_requests = Gauge::with_opts(
            Opts::new(
                "eligibility_active_requests",
                "Number of active unpaid leave eligibility evaluation requests"
            )
        ).unwrap();

        registry.register(Box::new(requests_total.clone())).unwrap();
        registry.register(Box::new(errors_total.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(active_requests.clone())).unwrap();

        EligibilityMetrics {
            registry,
            requests_total,
            errors_total,
            request_duration,
            active_requests,
        }
    }

    pub fn gather(&self) -> String {
        use prometheus::{Encoder, TextEncoder};
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

/// Timer struct to automatically measure request duration and track active requests
pub struct RequestTimer {
    timer: Option<prometheus::HistogramTimer>,
}

impl RequestTimer {
    pub fn new() -> Self {
        METRICS.active_requests.inc();
        let timer = METRICS.request_duration.start_timer();
        Self { timer: Some(timer) }
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        if let Some(timer) = self.timer.take() {
            timer.observe_duration();
        }
        METRICS.active_requests.dec();
    }
}

/// Helper function to increment request counter
pub fn increment_requests() {
    METRICS.requests_total.inc();
}

/// Helper function to increment error counter
pub fn increment_errors() {
    METRICS.errors_total.inc();
}
