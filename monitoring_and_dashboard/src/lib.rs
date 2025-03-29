use std::net::SocketAddr;
use std::convert::Infallible;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use tracing::{info, error};

/// Module de métriques : collecte et exposition des métriques au format Prometheus.
pub mod metrics {
    use prometheus::{Encoder, TextEncoder, Counter, Histogram, register_counter, register_histogram, gather};
    use lazy_static::lazy_static;

    lazy_static! {
        /// Compteur pour le nombre total d'événements.
        pub static ref MY_COUNTER: Counter = register_counter!(
            "nodara_events_total",
            "Nombre total d'événements traités par Nodara BIOSPHÈRE"
        ).expect("Échec de la création du compteur");

        /// Histogramme pour mesurer la durée de traitement des requêtes.
        pub static ref MY_HISTOGRAM: Histogram = register_histogram!(
            "nodara_request_duration_seconds",
            "Histogramme des durées de traitement des requêtes"
        ).expect("Échec de la création de l'histogramme");
    }

    /// Récupère toutes les métriques et les encode au format texte (exposition Prometheus).
    pub fn gather_metrics() -> String {
        let encoder = TextEncoder::new();
        let metric_families = gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)
            .expect("Échec de l'encodage des métriques");
        String::from_utf8(buffer)
            .expect("Les métriques ne sont pas en UTF-8")
    }
}

/// Module de dashboard : fonctions de gestion de la configuration des dashboards.
pub mod dashboard {
    use std::fs;
    use std::io;
    use tracing::info;

    /// Charge la configuration du dashboard à partir d'un fichier JSON.
    pub fn load_dashboard_config() -> io::Result<String> {
        let config = fs::read_to_string("grafana_dashboard.json")?;
        info!("Dashboard configuration loaded successfully.");
        Ok(config)
    }

    /// Fonction de rechargement de la configuration (peut être appelée par une API ou programmée périodiquement).
    pub fn reload_dashboard_config() -> io::Result<String> {
        load_dashboard_config()
    }
}

/// Initialise le module de monitoring et dashboard.
/// Cette fonction initialise la journalisation avec `tracing_subscriber` pour un logging structuré.
pub fn init_monitoring() {
    // Initialisation de la journalisation.
    tracing_subscriber::fmt::init();
    info!("Monitoring and Dashboard module initialized.");
}

/// Démarre un serveur HTTP pour exposer les métriques au format Prometheus.
pub async fn serve_metrics(addr: SocketAddr) {
    async fn metrics_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let body = metrics::gather_metrics();
        Ok(Response::new(Body::from(body)))
    }

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(metrics_handler))
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Serving metrics on http://{}", addr);

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}

/// Démarre un serveur HTTP pour exposer la configuration du dashboard.
/// Cela permet de recharger la configuration du dashboard via une API simple.
pub async fn serve_dashboard(addr: SocketAddr) {
    async fn dashboard_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        match dashboard::load_dashboard_config() {
            Ok(config) => Ok(Response::new(Body::from(config))),
            Err(e) => Ok(Response::new(Body::from(format!("Error loading dashboard config: {}", e)))),
        }
    }

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(dashboard_handler))
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Serving dashboard config on http://{}", addr);

    if let Err(e) = server.await {
        error!("Dashboard server error: {}", e);
    }
}
