use axum::Router;
use axum::extract::MatchedPath;
use axum::http::Request;
use blog_romira_dev_app::{
    App, AppState, SERVER_CONFIG, admin_routes, auth_routes, require_admin_auth, shell,
};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use sqlx::postgres::PgPoolOptions;
use time::macros::offset;
use tower::ServiceBuilder;
use tower::layer::util::{Identity, Stack};
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{MakeSpan, TraceLayer};
use tower_sessions::SessionManagerLayer;
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};
use tracing::Span;

#[tokio::main]
async fn main() {
    easy_init_newrelic_opentelemetry::NewRelicSubscriberInitializer::default()
        .newrelic_service_name("blog-romira-dev")
        .host_name(&SERVER_CONFIG.host_name)
        .newrelic_license_key(&SERVER_CONFIG.new_relic_license_key)
        .timestamps_offset(offset!(+09:00:00))
        .init()
        .expect("Failed to initialize NewRelic");

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
        .with_secure(false) // TODO: Set to true in production with HTTPS
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let app_state = AppState::new(leptos_options.clone(), db_pool);
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .merge(auth_routes())
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
        .layer(MakeSpanForHttp.into_tracing_service());

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

#[derive(Clone)]
struct MakeSpanForHttp;

impl<B> MakeSpan<B> for MakeSpanForHttp {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map_or(request.uri().to_string(), |m| m.as_str().to_string());
        tracing::info_span!(
            "request",
            http.method = %request.method(),
            http.uri = %request.uri(),
            http.version = ?request.version(),
            otel.name = format!("{} {}", request.method(), matched_path),
            otel.kind = "server",
        )
    }
}

impl MakeSpanForHttp {
    pub(crate) fn into_tracing_service(
        self,
    ) -> ServiceBuilder<Stack<TraceLayer<SharedClassifier<ServerErrorsAsFailures>, Self>, Identity>>
    {
        ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(self))
    }
}

/// Returns a layer that disables caching in development mode
#[cfg(debug_assertions)]
fn no_cache_layer() -> SetResponseHeaderLayer<http::HeaderValue> {
    SetResponseHeaderLayer::if_not_present(
        http::header::CACHE_CONTROL,
        http::HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    )
}
