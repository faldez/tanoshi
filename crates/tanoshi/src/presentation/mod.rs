#[cfg(feature = "embed")]
pub mod assets;
pub mod graphql;
pub mod rest;
pub mod token;

use anyhow::anyhow;
use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use graphql::schema::TanoshiSchema;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use self::{
    graphql::{graphql_handler, graphql_playground, loader::DatabaseLoader, schema::SchemaBuilder},
    rest::{health::health_check, image::fetch_image},
};
use crate::{
    application::worker::downloads::DownloadSender,
    db::MangaDatabase,
    domain::services::{
        image::ImageService, manga::MangaService, source::SourceService, tracker::TrackerService,
        user::UserService,
    },
    infrastructure::{
        notifier::Notifier,
        repositories::{
            image::ImageRepositoryImpl, manga::MangaRepositoryImpl, source::SourceRepositoryImpl,
            tracker::TrackerRepositoryImpl, user::UserRepositoryImpl,
        },
    },
};
use tanoshi_vm::extension::SourceBus;

pub struct ServerBuilder {
    user_svc: Option<UserService<UserRepositoryImpl>>,
    tracker_svc: Option<TrackerService<TrackerRepositoryImpl>>,
    source_svc: Option<SourceService<SourceRepositoryImpl>>,
    manga_svc: Option<MangaService<MangaRepositoryImpl>>,
    image_svc: Option<ImageService<ImageRepositoryImpl>>,
    mangadb: Option<MangaDatabase>,
    ext_manager: Option<SourceBus>,
    download_tx: Option<DownloadSender>,
    notifier: Option<Notifier<UserRepositoryImpl>>,
    enable_playground: bool,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            user_svc: None,
            tracker_svc: None,
            source_svc: None,
            manga_svc: None,
            image_svc: None,
            mangadb: None,
            ext_manager: None,
            download_tx: None,
            notifier: None,
            enable_playground: false,
        }
    }

    pub fn with_user_svc(self, user_svc: UserService<UserRepositoryImpl>) -> Self {
        Self {
            user_svc: Some(user_svc),
            ..self
        }
    }

    pub fn with_tracker_svc(self, tracker_svc: TrackerService<TrackerRepositoryImpl>) -> Self {
        Self {
            tracker_svc: Some(tracker_svc),
            ..self
        }
    }

    pub fn with_source_svc(self, source_svc: SourceService<SourceRepositoryImpl>) -> Self {
        Self {
            source_svc: Some(source_svc),
            ..self
        }
    }

    pub fn with_manga_svc(self, manga_svc: MangaService<MangaRepositoryImpl>) -> Self {
        Self {
            manga_svc: Some(manga_svc),
            ..self
        }
    }

    pub fn with_image_svc(self, image_svc: ImageService<ImageRepositoryImpl>) -> Self {
        Self {
            image_svc: Some(image_svc),
            ..self
        }
    }

    pub fn with_mangadb(self, mangadb: MangaDatabase) -> Self {
        Self {
            mangadb: Some(mangadb),
            ..self
        }
    }

    pub fn with_ext_manager(self, ext_manager: SourceBus) -> Self {
        Self {
            ext_manager: Some(ext_manager),
            ..self
        }
    }

    pub fn with_download_tx(self, download_tx: DownloadSender) -> Self {
        Self {
            download_tx: Some(download_tx),
            ..self
        }
    }

    pub fn with_notifier(self, notifier: Notifier<UserRepositoryImpl>) -> Self {
        Self {
            notifier: Some(notifier),
            ..self
        }
    }

    pub fn enable_playground(self) -> Self {
        Self {
            enable_playground: true,
            ..self
        }
    }

    pub fn build(self) -> Result<Server, anyhow::Error> {
        let user_svc = self.user_svc.ok_or_else(|| anyhow!("no user service"))?;
        let tracker_svc = self
            .tracker_svc
            .ok_or_else(|| anyhow!("no tracker service"))?;
        let source_svc = self
            .source_svc
            .ok_or_else(|| anyhow!("no source service"))?;
        let manga_svc = self.manga_svc.ok_or_else(|| anyhow!("no manga service"))?;
        let image_svc = self.image_svc.ok_or_else(|| anyhow!("no image service"))?;
        let mangadb = self.mangadb.ok_or_else(|| anyhow!("no manga database"))?;
        let extension_manager = self
            .ext_manager
            .ok_or_else(|| anyhow!("no extension manager"))?;
        let download_tx = self
            .download_tx
            .ok_or_else(|| anyhow!("no download sender"))?;
        let notifier = self.notifier.ok_or_else(|| anyhow!("no notifier"))?;

        let schema = SchemaBuilder::new()
            .data(user_svc)
            .data(tracker_svc)
            .data(source_svc)
            .data(manga_svc)
            .data(image_svc.clone())
            .data(mangadb.clone())
            .loader(DatabaseLoader { mangadb })
            .data(extension_manager)
            .data(download_tx)
            .data(notifier)
            .build();

        let enable_playground = self.enable_playground;

        Ok(Server::new(enable_playground, schema, image_svc))
    }
}

pub struct Server {
    router: Router<axum::body::Body>,
}

impl Server {
    pub fn new(
        enable_playground: bool,
        schema: TanoshiSchema,
        image_svc: ImageService<ImageRepositoryImpl>,
    ) -> Self {
        let mut router = Router::new();

        router = router
            .route("/health", get(health_check))
            .route("/image/:url", get(fetch_image))
            .layer(Extension(image_svc));

        if enable_playground {
            router = router
                .route("/graphql", get(graphql_playground).post(graphql_handler))
                .route("/graphql/", post(graphql_handler));
        } else {
            router = router
                .route("/graphql", post(graphql_handler))
                .route("/graphql/", post(graphql_handler));
        }

        router = router.layer(Extension(schema)).layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_credentials(true),
        );

        #[cfg(feature = "embed")]
        {
            router = router.fallback(get(assets::static_handler));
        }

        Self { router }
    }

    pub async fn serve<A: Into<SocketAddr>>(self, addr: A) -> Result<(), anyhow::Error> {
        axum::Server::bind(&addr.into())
            .serve(self.router.into_make_service())
            .await?;

        Ok(())
    }
}
