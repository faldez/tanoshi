use serde_json::json;

use std::sync::{Arc, RwLock};
use tanoshi_lib::extensions::Extension;
use tanoshi_lib::manga::{GetParams, Params, Source, SourceLogin};
use tanoshi_lib::rest::GetPagesResponse;
use warp::Rejection;

use std::io::Read;

use crate::auth::Claims;
use crate::extension::{repository::Repository, Extensions};
use crate::handlers::TransactionReject;

use std::convert::Infallible;
use warp::sse::ServerSentEvent;

#[derive(Clone)]
pub struct Manga {
    repo: Repository,
    exts: Arc<RwLock<Extensions>>,
}

impl Manga {
    pub fn new(database_path: String, exts: Arc<RwLock<Extensions>>) -> Self {
        Self {
            repo: Repository::new(database_path),
            exts,
        }
    }

    pub async fn list_sources(&self, param: String) -> Result<impl warp::Reply, Rejection> {
        match param.as_str() {
            "available" => {
                let resp = ureq::get(
                    format!("https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo-{}/index.json", std::env::consts::OS).as_str(),
                )
                .call();
                let sources = resp.into_json_deserialize::<Vec<Source>>().unwrap();
                Ok(warp::reply::json(&json!(
                    {
                        "sources": sources,
                        "status": "success"
                    }
                )))
            }
            "installed" => {
                let exts = self.exts.read().unwrap();
                let sources = exts
                    .extensions()
                    .iter()
                    .map(|(_key, ext)| ext.info())
                    .collect::<Vec<Source>>();

                match self.repo.insert_sources(sources) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("error get source {}", e.to_string());
                        return Err(warp::reject());
                    }
                }

                match self.repo.get_sources() {
                    Ok(sources) => {
                        debug!("sources {:?}", sources.clone());
                        Ok(warp::reply::json(&json!(
                            {
                                "sources": sources,
                                "status": "success"
                            }
                        )))
                    }
                    Err(e) => {
                        error!("error get source {}", e.to_string());
                        Err(warp::reject::custom(TransactionReject {
                            message: e.to_string(),
                        }))
                    }
                }
            }
            _ => Err(warp::reject()),
        }
    }

    pub async fn install_source(
        &self,
        name: String,
        plugin_path: String,
    ) -> Result<impl warp::Reply, Rejection> {
        let ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else if cfg!(target_os = "linux") {
            "so"
        } else {
            return Err(warp::reject());
        };

        let resp = ureq::get(
            format!(
                "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo-{}/library/lib{}.{}",
                std::env::consts::OS,
                name.clone(),
                ext.clone(),
            )
            .as_str(),
        )
        .call();
        let mut reader = resp.into_reader();
        let mut bytes = vec![];
        if let Err(e) = reader.read_to_end(&mut bytes) {
            return Err(warp::reject::custom(TransactionReject {
                message: e.to_string(),
            }));
        }

        let path = std::path::PathBuf::from(plugin_path);
        let path = path.join(format!("lib{}.{}", name, ext));
        if let Err(e) = std::fs::write(path.clone(), &bytes) {
            return Err(warp::reject::custom(TransactionReject {
                message: e.to_string(),
            }));
        }

        Ok(warp::reply())
    }

    pub async fn list_mangas(
        &self,
        source_id: i32,
        claim: Claims,
        source_auth: String,
        param: Params,
    ) -> Result<impl warp::Reply, Rejection> {
        let exts = self.exts.read().unwrap();
        if let Ok(source) = self.repo.get_source(source_id) {
            let mangas = exts
                .get(&source.name)
                .unwrap()
                .get_mangas(&source.url, param, source_auth)
                .unwrap();
            debug!("mangas {:?}", mangas.clone());

            let manga_ids = match self.repo.insert_mangas(source_id, mangas.clone()) {
                Ok(ids) => {
                    debug!("manga ids {:?}", ids);
                    ids
                }
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }));
                }
            };
            match self.repo.get_mangas(claim.sub, manga_ids) {
                Ok(mangas) => return Ok(warp::reply::json(&mangas)),
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }));
                }
            }
        }
        Err(warp::reject())
    }

    pub async fn get_manga_info(
        &self,
        manga_id: i32,
        claim: Claims,
    ) -> Result<impl warp::Reply, Rejection> {
        let exts = self.exts.read().unwrap();
        if let Ok(manga) = self.repo.get_manga_detail(manga_id, claim.sub.clone()) {
            return Ok(warp::reply::json(&manga));
        } else if let Ok(url) = self.repo.get_manga_url(manga_id) {
            let source = match self.repo.get_source_from_manga_id(manga_id) {
                Ok(source) => source,
                Err(e) => {
                    error!("error get_manga_url {}", e.to_string());
                    return Err(warp::reject());
                }
            };

            let manga = exts
                .get(&source.name)
                .unwrap()
                .get_manga_info(&url)
                .unwrap();

            match self.repo.update_manga_info(manga_id, manga) {
                Ok(_) => {}
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }));
                }
            }

            match self.repo.get_manga_detail(manga_id, claim.sub) {
                Ok(res) => return Ok(warp::reply::json(&res)),
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }));
                }
            }
        }
        Err(warp::reject())
    }

    pub async fn get_chapters(
        &self,
        manga_id: i32,
        claim: Claims,
        param: GetParams,
    ) -> Result<impl warp::Reply, Rejection> {
        let exts = self.exts.read().unwrap();
        let refresh = param.refresh.unwrap_or(false);
        if !refresh {
            match self.repo.get_chapters(manga_id, claim.sub.clone()) {
                Ok(chapter) => return Ok(warp::reply::json(&chapter)),
                Err(_e) => {}
            };
        }

        if let Ok(url) = self.repo.get_manga_url(manga_id) {
            let source = match self.repo.get_source_from_manga_id(manga_id) {
                Ok(source) => source,
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }))
                }
            };

            let chapter = match exts.get(&source.name).unwrap().get_chapters(&url) {
                Ok(ch) => ch,
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }))
                }
            };

            match self
                .repo
                .insert_chapters(claim.sub.clone(), manga_id, chapter.clone())
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }));
                }
            }

            match self.repo.get_chapters(manga_id, claim.sub) {
                Ok(chapter) => return Ok(warp::reply::json(&chapter)),
                Err(e) => {
                    return Err(warp::reject::custom(TransactionReject {
                        message: e.to_string(),
                    }));
                }
            };
        }
        Err(warp::reject())
    }

    pub async fn get_pages(
        &self,
        chapter_id: i32,
        param: GetParams,
    ) -> anyhow::Result<GetPagesResponse> {
        let exts = self.exts.read().unwrap();

        let refresh = param.refresh.unwrap_or(false);
        if refresh {
            if let Err(e) = self.repo.delete_pages(chapter_id) {
                error!("error delete page: {}", e);
            }
        }

        if let Ok(pages) = self.repo.get_pages(chapter_id) {
            return Ok(pages);
        };

        if let Ok(url) = self.repo.get_chapter_url(chapter_id) {
            let source = match self.repo.get_source_from_chapter_id(chapter_id) {
                Ok(source) => source,
                Err(e) => return Err(anyhow::anyhow!("{}", e.to_string())),
            };

            let pages = exts.get(&source.name).unwrap().get_pages(&url).unwrap();

            match self.repo.insert_pages(chapter_id, pages.clone()) {
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow::anyhow!("{}", e.to_string()));
                }
            }

            match self.repo.get_pages(chapter_id) {
                Ok(pages) => return Ok(pages),
                Err(e) => {
                    return Err(anyhow::anyhow!("{}", e.to_string()));
                }
            };
        }
        Err(anyhow::anyhow!("pages not found"))
    }

    pub async fn proxy_image(&self, page_id: i32) -> Result<impl warp::Reply, Rejection> {
        let image = match self.repo.get_image_from_page_id(page_id) {
            Ok(image) => image,
            Err(_) => return Err(warp::reject()),
        };

        let exts = self.exts.read().unwrap();
        let bytes = exts
            .get(&image.source_name)
            .unwrap()
            .get_page(&image.url)
            .unwrap();

        let path = std::path::PathBuf::from(image.path).join(image.file_name);
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        let resp = warp::http::Response::builder()
            .header("Content-Type", mime.as_ref())
            .header("Content-Length", bytes.len())
            .body(bytes)
            .unwrap();

        Ok(resp)
    }

    pub async fn source_login(
        &self,
        source_id: i32,
        login_info: SourceLogin,
    ) -> Result<impl warp::Reply, Rejection> {
        let exts = self.exts.read().unwrap();
        if let Ok(source) = self.repo.get_source(source_id) {
            if let Ok(result) = exts.get(&source.name).unwrap().login(login_info) {
                let mut result = result;
                result.source_id = source_id;
                return Ok(warp::reply::json(&result));
            }
        }
        Err(warp::reject())
    }

    pub fn get_image(&self, page_id: i32) -> Result<impl ServerSentEvent, Infallible> {
        if page_id == -1 {
            return Ok(warp::sse::data("done".to_string()));
        }
        let image = self.repo.get_image_from_page_id(page_id).unwrap();

        let exts = self.exts.read().unwrap();
        let _ = exts
            .get(&image.source_name)
            .unwrap()
            .get_page(&image.url)
            .unwrap();

        Ok(warp::sse::data(format!("/api/page/{}", page_id)))
    }
}
