use anyhow::Context;
use axum::{routing, Router};
use navigator::html::{homepage, navigator, visions, send_note, check_in, about};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    info!("Launching the Guild Navigator...");
    
    // Get route to self
    let src_path = std::env::current_dir().unwrap();

    //    // Microservices to interact with relays
    //    let api_router = Router::new()
    //        .route("/notebook", routing::post(notebook))

    let port = 6900_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    // info!("Assets path: {:?}", assets_path.to_str().unwrap());
    let router = Router::new()
        .route("/", routing::get(homepage))
        .route("/login", routing::get(check_in))
        .route("/about", routing::get(about))
        .route("/checkIn", routing::post(navigator))
        .route("/filterRequest", routing::post(visions))
        .route("/sendNote", routing::post(send_note))
        .nest_service(
            "/js",
            ServeDir::new(format!("{}/public/js", src_path.to_str().unwrap())),
        )
        .nest_service(
            "/styles",
            ServeDir::new(format!("{}/public/styles", src_path.to_str().unwrap())),
        );
    info!("The Guild Navigator is in tank {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("Folding internet-space...")
        .expect("Not enough melange in the tank.");
    Ok(())
}
