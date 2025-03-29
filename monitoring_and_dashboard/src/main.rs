//! # Monitoring and Dashboard - Main Entry Point
//!
//! Ce binaire démarre un serveur HTTP qui expose les métriques collectées par le module.
//! Vous pouvez adapter le port et la logique selon vos besoins.

use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use tokio;

mod metrics {
    use lazy_static::lazy_static;
    use prometheus::{register_counter, register_histogram, Encoder, TextEncoder, Counter, Histogram, gather};

    lazy_static! {
        /// Compteur pour le nombre total d'événements.
        pub static ref EVENTS_TOTAL: Counter = register_counter!(
            "nodara_events_total",
            "Nombre total d'événements traités par Nodara BIOSPHÈRE"
        ).expect("Échec de la création du compteur");

        /// Histogramme pour mesurer la durée de traitement.
        pub static ref REQUEST_DURATION: Histogram = register_histogram!(
            "nodara_request_duration_seconds",
            "Durée de traitement des requêtes"
        ).expect("Échec de la création de l'histogramme");
    }

    /// Rassemble et encode les métriques en format texte pour Prometheus.
    pub fn gather_metrics() -> String {
        let encoder = TextEncoder::new();
        let metric_families = gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).expect("Échec de l'encodage des métriques");
        String::from_utf8(buffer).expect("Les métriques ne sont pas en UTF-8")
    }
}

async fn metrics_handler(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let body = metrics::gather_metrics();
    Ok(Response::new(Body::from(body)))
}

/// Démarre le serveur HTTP qui expose les métriques.
async fn serve_metrics(addr: SocketAddr) {
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(metrics_handler))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

#[tokio::main]
async fn main() {
    // Initialiser le logger
    env_logger::init();

    println!("Monitoring and Dashboard module starting...");

    // Exemple d'incrémentation d'une métrique
    metrics::EVENTS_TOTAL.inc();

    // Démarrer le serveur sur le port 9898
    let addr: SocketAddr = "127.0.0.1:9898".parse().expect("Adresse invalide");
    serve_metrics(addr).await;
}
