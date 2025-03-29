use std::sync::Arc;

use axum::{extract::{Request, State}, middleware::Next, response::Response};
use prometheus::{Counter, Encoder, Opts, Registry, TextEncoder};
use reqwest::StatusCode;
use tracing::{event, Level};

use crate::state::AppState;

pub trait MetricsService {
    fn register_metric(&mut self, metric_name: String, metric_help_text: String, metric_type: MetricType);
}

pub struct PrometheusMetricsService {
}

impl PrometheusMetricsService {
    pub fn new() -> PrometheusMetricsService {
        PrometheusMetricsService {  }
    }
}

impl MetricsService for PrometheusMetricsService {
    fn register_metric(&mut self, metric_name: String, metric_help_text: String, metric_type: MetricType) {
        let metric_opts = Opts::new(metric_name, metric_help_text);

        match metric_type {
            MetricType::COUNTER => {
                match Counter::with_opts(metric_opts) {
                    Ok(counter) => {
                        let registry = Registry::new();
                        registry.register(Box::new(counter.clone())).unwrap();

                        counter.inc();

                        let mut buffer = vec![];
                        let encoder = TextEncoder::new();
                        let metric_families = registry.gather();
                        encoder.encode(&metric_families, &mut buffer).unwrap();

                        println!("{}", String::from_utf8(buffer).unwrap());
                    },
                    Err(e) => {
                        event!(Level::WARN, "Error occurred while creating counter metric: {}", e)
                    }
                }
            }
        }
    }
}

pub enum MetricType {
    COUNTER,
}