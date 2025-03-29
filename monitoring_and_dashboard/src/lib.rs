//! # Monitoring and Dashboard Module
//!
//! Ce module fournit des fonctionnalités pour la surveillance et l'intégration de dashboards dans Nodara BIOSPHÈRE QUANTIC.
//! Il inclut :
//! - L'initialisation du logging et de la collecte de métriques.
//! - Un serveur HTTP pour exposer les métriques au format Prometheus.
//! - Des fonctions pour charger une configuration de dashboard (ex. Grafana).
//!
//! ## Exemples d'utilisation
//!
//! Initialisez le module et démarrez le serveur de métriques :
//!
//! ```no_run
//! use monitoring_and_dashboard::{init_monitoring, serve_metrics, dashboard, metrics};
//! use std::net::SocketAddr;
//! 
//! #[tokio::main]
//! async fn main() {
//!     // Initialisation du module
//!     init_monitoring();
//!     
//!     // Incrémente un compteur exemple
//!     metrics::MY_COUNTER.inc();
//!     
//!     // Chargement de la configuration du dashboard
//!     match dashboard::load_dashboard_config() {
//!         Ok(config) => println!("Dashboard config: {}", config),
//!         Err(e) => eprintln!("Erreur de chargement de la config: {}", e),
//!     }
//!     
//!     // Démarrage du serveur HTTP pour exposer les métriques
//!     let addr: SocketAddr = "127.0.0.1:9898".parse().expect("Adresse invalide");
//!     serve_metrics(addr).await;
//! }
//! ```  

use std::net::SocketAddr;
use std::convert::Infallible;

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

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

    /// Charge la configuration du dashboard à partir d'un fichier JSON.
    pub fn load_dashboard_config() -> io::Result<String> {
        fs::read_to_string("grafana_dashboard.json")
    }
}

/// Initialise le module de monitoring et dashboard.
/// Cette fonction configure le logging et peut être étendue pour d'autres initialisations.
pub fn init_monitoring() {
    // Initialisation du logging (ignore l'erreur si déjà initialisé)
    let _ = env_logger::builder().is_test(true).try_init();
    println!("Monitoring and Dashboard module initialized.");
}

/// Démarre un serveur HTTP pour exposer les métriques Prometheus.
/// 
/// # Arguments
/// 
/// * `addr` - L'adresse socket sur laquelle le serveur écoutera.
/// 
/// # Exemple
/// 
/// ```no_run
/// use monitoring_and_dashboard::serve_metrics;
/// use std::net::SocketAddr;
/// 
/// #[tokio::main]
/// async fn main() {
///     let addr: SocketAddr = "127.0.0.1:9898".parse().unwrap();
///     serve_metrics(addr).await;
/// }
/// ```
pub async fn serve_metrics(addr: SocketAddr) {
    async fn metrics_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let body = metrics::gather_metrics();
        Ok(Response::new(Body::from(body)))
    }

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(metrics_handler))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Serving metrics on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
