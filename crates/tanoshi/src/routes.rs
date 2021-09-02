#[macro_export]
macro_rules! bind_routes {
    ($port:expr, $route:ident, $($routes:ident),+) => {{
        let cors = warp::cors().allow_any_origin().allow_method("POST");
        let routes = $route
        $(.or($routes))+
        .recover(|err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        })
        .with(cors);

        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], $port as u16), async {
            let sigint_signal = tokio::signal::ctrl_c();

            #[cfg(target_family = "windows")]
            sigint_signal.await.expect("failed listening ctrl_c");

            #[cfg(target_family = "unix")]
            {
                let mut stream = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("failed to create stream");
                tokio::select! {
                    Some(_) = stream.recv() => {
                        info!("sigterm");
                    }
                    _ = sigint_signal => {
                        info!("sigint");
                    }
                }
            }
       });

       tokio::spawn(server)
    }};
}
