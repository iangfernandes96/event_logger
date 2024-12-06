// // src/main.rs

// use rand::prelude::SliceRandom;
// use rand::{distributions::Alphanumeric, Rng};
// use scylla::{batch::Batch, Session, SessionBuilder};
// use std::sync::Arc;
// use std::time::{SystemTime, UNIX_EPOCH};
// use tokio; // Import SliceRandom trait

// #[derive(Debug)]
// struct Event {
//     event_id: uuid::Uuid,
//     event_type: String,
//     timestamp: i64,
//     payload: String,
// }

// fn generate_event() -> Event {
//     let event_id = uuid::Uuid::new_v4();
//     let event_type = ["login", "purchase", "logout"]
//         .choose(&mut rand::thread_rng())
//         .unwrap()
//         .to_string();
//     let timestamp = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_millis() as i64;
//     let payload: String = rand::thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(100)
//         .map(char::from)
//         .collect();

//     Event {
//         event_id,
//         event_type,
//         timestamp,
//         payload,
//     }
// }

// async fn create_keyspace_and_table(session: &Session) {
//     // Create keyspace if it doesn't exist
//     let create_keyspace_query = "
//         CREATE KEYSPACE IF NOT EXISTS event_logging
//         WITH REPLICATION = {'class': 'SimpleStrategy', 'replication_factor': 1};;
//     ";
//     session
//         .query(create_keyspace_query, ())
//         .await
//         .expect("Failed to create keyspace");

//     // Create events table if it doesn't exist
//     let create_table_query = "
//         CREATE TABLE IF NOT EXISTS event_logging.events (
//             event_id UUID,
//             event_type TEXT,
//             timestamp TIMESTAMP,
//             payload TEXT,
//             PRIMARY KEY (event_id, timestamp)) WITH CLUSTERING ORDER BY (timestamp DESC);
//     ";
//     session
//         .query(create_table_query, ())
//         .await
//         .expect("Failed to create events table");
//     // Create index on event_type if it doesn't exist
//     let create_index_query = "CREATE INDEX IF NOT EXISTS ON event_logging.events (event_type);";
//     session
//         .query(create_index_query, ())
//         .await
//         .expect("Failed to create index on event_type");
// }

// async fn ingest_events(session: Arc<Session>, num_events: usize) {
//     for _ in 0..num_events {
//         let event = generate_event();
//         let query =
//             "INSERT INTO event_logging.events (event_id, event_type, timestamp, payload) VALUES (?, ?, ?, ?)";
//         session
//             .query(
//                 query,
//                 (
//                     event.event_id,
//                     event.event_type,
//                     event.timestamp,
//                     event.payload,
//                 ),
//             )
//             .await
//             .expect("Failed to ingest event");
//     }
// }

// async fn count_events_by_type(session: Arc<Session>, event_type: &str) {
//     let query = "SELECT COUNT(*) FROM event_logging.events WHERE event_type = ?";
//     let result = session
//         .query(query, (event_type,))
//         .await
//         .expect("Query failed");

//     // Check if rows are present
//     if let Some(rows) = result.rows {
//         let count: i64 = rows[0].columns[0].as_ref().unwrap().as_bigint().unwrap();
//         println!("Event type '{}': {} events", event_type, count);
//     } else {
//         println!("No rows found for event type '{}'", event_type);
//     }
// }

// async fn fetch_recent_events(session: Arc<Session>, event_type: &str, limit: usize) {
//     let query =
//         "SELECT event_id, timestamp, payload FROM event_logging.events WHERE event_type = ? LIMIT ?";
//     let result = session
//         .query(query, (event_type, limit as i32))
//         .await
//         .expect("Query failed");

//     // Check if rows are present
//     if let Some(rows) = result.rows {
//         for row in rows {
//             let event_id: uuid::Uuid = row.columns[0].as_ref().unwrap().as_uuid().unwrap();
//             let timestamp: i64 = row.columns[1].as_ref().unwrap().as_bigint().unwrap();
//             let payload: String = row.columns[2]
//                 .as_ref()
//                 .unwrap()
//                 .as_text()
//                 .unwrap()
//                 .to_string();
//             println!("{} | {} | {}", event_id, timestamp, payload);
//         }
//     } else {
//         println!("No rows found for event type '{}'", event_type);
//     }
// }

// #[tokio::main]
// async fn main() {
//     let session = Arc::new(
//         SessionBuilder::new()
//             .known_node("127.0.0.1:9042")
//             .build()
//             .await
//             .expect("Failed to connect to ScyllaDB"),
//     );
//     create_keyspace_and_table(&session).await;

//     // Ingest events
//     let mut tasks = vec![];
//     for _ in 0..1000 {
//         let session_clone = session.clone();
//         tasks.push(tokio::spawn(async move {
//             ingest_events(session_clone, 1000).await;
//         }));
//     }

//     for task in tasks {
//         task.await.unwrap();
//     }

//     println!("Ingestion complete");

//     // Query analytics
//     count_events_by_type(session.clone(), "login").await;
//     fetch_recent_events(session.clone(), "purchase", 5).await;
// }

// src/main.rs

// src/main.rs

// src/main.rs