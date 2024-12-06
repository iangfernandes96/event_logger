use env_logger::Builder;
use log::LevelFilter;
use prometheus::{
    register_gauge, register_int_counter, Encoder, Gauge, IntCounter, Registry, TextEncoder,
};
use rand::{distributions::Alphanumeric, Rng};
use scylla::{Session, SessionBuilder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio;
use ulid::Ulid;
use warp::Filter;

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    event_id: Ulid,
    event_type: String,
    timestamp: i64,
    payload: String,
}

lazy_static::lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    static ref API_REQUESTS: IntCounter = register_int_counter!("api_requests_total", "Total number of API requests").unwrap();
    static ref API_REQUEST_LATENCY: Gauge = register_gauge!("api_request_latency_seconds", "API request latency in seconds").unwrap();
    static ref API_EXCEPTIONS: IntCounter = register_int_counter!("api_exceptions_total", "Total number of API exceptions").unwrap();
}

fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(API_REQUESTS.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(API_REQUEST_LATENCY.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(API_EXCEPTIONS.clone()))
        .expect("collector can be registered");
}

fn generate_event(event_type: &str) -> Event {
    let event_id = Ulid::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let payload: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(100)
        .map(char::from)
        .collect();

    Event {
        event_id,
        event_type: event_type.to_string(),
        timestamp,
        payload,
    }
}

// Metrics endpoint
async fn metrics() -> Result<impl warp::Reply, warp::Rejection> {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(warp::reply::with_header(
        buffer,
        "Content-Type",
        encoder.format_type(),
    ))
}

async fn create_keyspace_and_table(session: &Session) {
    // Create keyspace if it doesn't exist with replication factor of 1
    let create_keyspace_query = "
        CREATE KEYSPACE IF NOT EXISTS event_logging 
        WITH REPLICATION = {'class': 'SimpleStrategy', 'replication_factor': 1};
    ";
    session
        .query(create_keyspace_query, ())
        .await
        .expect("Failed to create keyspace");

    // Create events table if it doesn't exist
    let create_table_query = "
        CREATE TABLE IF NOT EXISTS event_logging.events (
            event_id TEXT,
            event_type TEXT,
            timestamp TIMESTAMP,
            payload TEXT,
            PRIMARY KEY (event_type, event_id, timestamp)) WITH CLUSTERING ORDER BY (event_id ASC, timestamp DESC);
    ";
    session
        .query(create_table_query, ())
        .await
        .expect("Failed to create events table");
}

async fn ingest_event(session: Arc<Session>, event: Event) {
    let query = "INSERT INTO event_logging.events (event_id, event_type, timestamp, payload) VALUES (?, ?, ?, ?)";
    session
        .query(
            query,
            (
                event.event_id.to_string(),
                event.event_type,
                event.timestamp,
                event.payload,
            ),
        )
        .await
        .expect("Failed to ingest event");
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_timestamp_secs()
        .init();
    // register_custom_metrics();
    let session = Arc::new(
        SessionBuilder::new()
            .known_node("127.0.0.1:9042")
            .build()
            .await
            .expect("Failed to connect to ScyllaDB"),
    );

    // Create keyspace and table if they don't exist
    create_keyspace_and_table(&session).await;

    // Define the HTTP route for ingesting events
    let ingest_route = warp::post()
        .and(warp::path("ingest"))
        .and(warp::body::json())
        .map({
            let session_clone = session.clone();
            move |event: Event| {
                let session_clone = session_clone.clone();
                tokio::spawn(async move {
                    API_REQUESTS.inc();
                    let start_time = std::time::Instant::now(); // Start timing
                    ingest_event(session_clone, event).await;
                    let duration = start_time.elapsed().as_secs_f64();
                    API_REQUEST_LATENCY.set(duration);
                });
                warp::reply::with_status("Event ingested", warp::http::StatusCode::OK)
            }
        });

    let metrics_route = warp::path("metrics").and_then(metrics);

    // Start the HTTP server
    let routes = ingest_route.or(metrics_route);
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030)) // Listen on localhost:3030
        .await;
}
