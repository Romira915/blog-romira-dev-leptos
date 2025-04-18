use axum::Router;
use axum::extract::MatchedPath;
use axum::http::Request;
use blog_romira_dev_app::*;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use time::macros::offset;
use tower::ServiceBuilder;
use tower::layer::util::{Identity, Stack};
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, TraceLayer};
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

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let app_state = AppState::new(leptos_options.clone());
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
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
        .layer(MakeSpanForHttp.into_tracing_service());

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
