use axum::Router;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use blog_romira_dev_app::{
    App, AppState, SERVER_CONFIG, admin_routes, auth_routes, dbsc_routes, require_admin_auth,
    seo_routes, shell,
};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use opentelemetry::KeyValue;
use sqlx::postgres::PgPoolOptions;
use std::sync::LazyLock;
use std::time::Instant;
use time::macros::offset;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};
use tracing::Instrument;

#[tokio::main]
async fn main() {
    let mut telemetry_init =
        easy_init_newrelic_opentelemetry::NewRelicSubscriberInitializer::default()
            .newrelic_service_name(&SERVER_CONFIG.new_relic_service_name)
            .host_name(&SERVER_CONFIG.host_name)
            .newrelic_license_key(&SERVER_CONFIG.new_relic_license_key)
            .timestamps_offset(offset!(+09:00:00));
    if !SERVER_CONFIG.otlp_endpoint.is_empty() {
        telemetry_init = telemetry_init.newrelic_otlp_endpoint(&SERVER_CONFIG.otlp_endpoint);
    }
    let _telemetry_guard = telemetry_init
        .init()
        .expect("Failed to initialize OpenTelemetry");

    // Database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&SERVER_CONFIG.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("../migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    // Session store (Valkey)
    let valkey_config = Config::from_url(&SERVER_CONFIG.valkey_url).expect("Invalid Valkey URL");
    let valkey_pool =
        Pool::new(valkey_config, None, None, None, 6).expect("Failed to create Valkey pool");
    valkey_pool.connect();
    valkey_pool
        .wait_for_connect()
        .await
        .expect("Failed to connect to Valkey");
    let session_store = RedisStore::new(valkey_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(tower_sessions::Expiry::OnInactivity(time::Duration::weeks(
            2,
        )));

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let app_state = AppState::new(leptos_options.clone(), db_pool);
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .merge(seo_routes())
        .merge(auth_routes())
        .merge(dbsc_routes())
        .merge(admin_routes())
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                use blog_romira_dev_app::common::response::CacheControlSet;
                let app_state = app_state.clone();
                move || {
                    provide_context(app_state.clone());
                    provide_context(CacheControlSet::new());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        // .leptos_routes(&app_state, routes, {
        //     let leptos_options = leptos_options.clone();
        //     move || shell(leptos_options.clone())
        // })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state)
        .layer(axum::middleware::from_fn(require_admin_auth))
        .layer(session_layer)
        .layer(axum::middleware::from_fn(http_observability));

    // Disable caching in development mode
    #[cfg(debug_assertions)]
    let app = app.layer(no_cache_layer());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

static HTTP_SERVER_REQUEST_DURATION: LazyLock<opentelemetry::metrics::Histogram<f64>> =
    LazyLock::new(|| {
        opentelemetry::global::meter("http")
            .f64_histogram("http.server.request.duration")
            .with_description("Duration of HTTP server requests.")
            .with_unit("s")
            .build()
    });

async fn http_observability(req: Request<axum::body::Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "UNMATCHED".to_string());

    let span = tracing::info_span!(
        "request",
        http.request.method = %method,
        http.route = %route,
        http.response.status_code = tracing::field::Empty,
        otel.name = format!("{} {}", method, route),
        otel.kind = "server",
    );

    let response = next.run(req).instrument(span.clone()).await;

    let status = response.status().as_u16();
    span.record("http.response.status_code", i64::from(status));

    let duration = start.elapsed().as_secs_f64();
    HTTP_SERVER_REQUEST_DURATION.record(
        duration,
        &[
            KeyValue::new("http.request.method", method.to_string()),
            KeyValue::new("http.route", route),
            KeyValue::new("http.response.status_code", i64::from(status)),
        ],
    );

    response
}

/// Returns a layer that disables caching in development mode
#[cfg(debug_assertions)]
fn no_cache_layer() -> SetResponseHeaderLayer<http::HeaderValue> {
    SetResponseHeaderLayer::if_not_present(
        http::header::CACHE_CONTROL,
        http::HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    )
}
