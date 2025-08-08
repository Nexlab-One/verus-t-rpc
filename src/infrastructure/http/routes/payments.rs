//! Payments routes

use std::sync::Arc;
use warp::Filter;

use crate::application::services::payments_service::PaymentsService;
use crate::config::AppConfig;
use crate::infrastructure::http::handlers::{handle_payment_quote, handle_payment_status, handle_payment_submit};

pub struct PaymentsRoutes;

impl PaymentsRoutes {
    pub fn create_routes(
        config: AppConfig,
        service: Arc<PaymentsService>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let quote = warp::path("payments")
            .and(warp::path("request"))
            .and(warp::post())
            .and(warp::body::content_length_limit(config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(Self::with_service(service.clone()))
            .and(Self::with_config(config.clone()))
            .and_then(handle_payment_quote);

        let submit = warp::path("payments")
            .and(warp::path("submit"))
            .and(warp::post())
            .and(warp::body::content_length_limit(config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(Self::with_service(service.clone()))
            .and(Self::with_config(config.clone()))
            .and_then(handle_payment_submit);

        let status = warp::path("payments")
            .and(warp::path("status"))
            .and(warp::path::param::<String>())
            .and(warp::get())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(Self::with_service(service))
            .and(Self::with_config(config))
            .and_then(handle_payment_status);

        quote.or(submit).or(status)
    }

    fn with_service(
        service: Arc<PaymentsService>,
    ) -> impl Filter<Extract = (Arc<PaymentsService>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || service.clone())
    }

    fn with_config(
        config: AppConfig,
    ) -> impl Filter<Extract = (AppConfig,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || config.clone())
    }
}


