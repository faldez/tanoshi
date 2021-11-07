use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};
use tanoshi_lib::prelude::{Chapter, Extension, ExtensionResult, Filters, Manga, Param, Source};
use tanoshi_util::http::Request;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
    time::Instant,
};
#[cfg(feature = "compiler")]
use wasmer::Target;
use wasmer::{imports, ChainableNamedResolver, Function, Instance, Module, Store, WasmerEnv};

use wasmer_wasi::{Pipe, WasiEnv, WasiState};

use crate::bus::{Command, ExtensionResultSender};

#[derive(WasmerEnv, Clone)]
struct ExtensionEnv {
    wasi_env: WasiEnv,
}

pub struct ExtensionProxy {
    path: PathBuf,
}

impl ExtensionProxy {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<P: AsRef<Path>>(path: P) -> Arc<dyn Extension> {
        let path = PathBuf::new().join(path);
        Arc::new(Self { path })
    }

    fn load(&self) -> Result<(Instance, ExtensionEnv), Box<dyn std::error::Error>> {
        let instant = Instant::now();
        let store = Self::init_store_headless();
        let module = unsafe { Module::deserialize_from_file(&store, &self.path)? };

        let stdin = Pipe::new();
        let stdout = Pipe::new();
        let stderr = Pipe::new();

        let mut wasi_env = WasiState::new("tanoshi")
            .stdin(Box::new(stdin))
            .stdout(Box::new(stdout))
            .stderr(Box::new(stderr))
            .finalize()?;

        let import_object = wasi_env.import_object(&module)?;

        let env = ExtensionEnv { wasi_env };

        let tanoshi = imports! {
            "tanoshi" => {
                "host_http_request" => Function::new_native_with_env(&store, env.clone(), host_http_request),
                "host_debug" => Function::new_native_with_env(&store, env.clone(), host_debug),
                "host_error" => Function::new_native_with_env(&store, env.clone(), host_error),
                "host_info" => Function::new_native_with_env(&store, env.clone(), host_info),
                "host_trace" => Function::new_native_with_env(&store, env.clone(), host_trace),
                "host_warn" => Function::new_native_with_env(&store, env.clone(), host_warn),
            }
        };

        let instance = Instance::new(&module, &tanoshi.chain_back(import_object))?;
        debug!("extension loaded in {} ms", instant.elapsed().as_millis());
        Ok((instance, env))
    }

    #[cfg(feature = "compiler")]
    fn init_store(target: Target) -> Store {
        let compiler = wasmer_compiler_llvm::LLVM::new();

        let engine = wasmer_engine_dylib::Dylib::new(compiler)
            .target(target)
            .engine();

        Store::new(&engine)
    }

    fn init_store_headless() -> Store {
        let engine = wasmer_engine_dylib::Dylib::headless().engine();

        Store::new(&engine)
    }

    #[cfg(feature = "compiler")]
    pub fn compile_from_file<P: AsRef<Path>>(
        path: P,
        target: Target,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let wasm_bytes = std::fs::read(&path)?;

        let output = std::path::PathBuf::new()
            .join(&path)
            .with_extension(format!("{}.tanoshi", target.triple()));
        Self::compile(&wasm_bytes, output, target)?;

        Ok(())
    }

    #[cfg(feature = "compiler")]
    pub fn compile<P: AsRef<Path>>(
        wasm_bytes: &[u8],
        output: P,
        target: Target,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = Self::init_store(target);

        debug!("compiling extension");
        let module = Module::new(&store, wasm_bytes)?;
        debug!("done");

        Ok(module.serialize_to_file(output)?)
    }

    fn call<T>(&self, name: &str) -> Result<T, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned,
    {
        let (instance, env) = self.load()?;
        let res = instance.exports.get_function(name)?;
        res.call(&[])?;
        let object_str = wasi_read(&env)?;
        debug!("call {} => {}", name, object_str);
        Ok(ron::from_str(&object_str)?)
    }

    fn call_with_args<T, U>(&self, name: &str, param: &U) -> Result<T, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned,
        U: Serialize + Debug,
    {
        let (instance, env) = self.load()?;
        let res = instance.exports.get_function(name)?;
        if let Err(e) = wasi_write(&env, &param) {
            error!("error write to wasi: {}", e);
        }
        res.call(&[])?;
        let object_str = wasi_read(&env)?;
        debug!("call {}({:?}) => {}", name, param, object_str);
        Ok(ron::from_str(&object_str)?)
    }
}

impl Extension for ExtensionProxy {
    fn detail(&self) -> Source {
        self.call("detail").unwrap_or_default()
    }

    fn filters(&self) -> ExtensionResult<Option<Filters>> {
        match self.call("filters") {
            Ok(res) => res,
            Err(e) => ExtensionResult::err(format!("{}", e).as_str()),
        }
    }

    fn get_manga_list(&self, param: Param) -> ExtensionResult<Vec<Manga>> {
        match self.call_with_args("get_manga_list", &param) {
            Ok(res) => res,
            Err(e) => ExtensionResult::err(format!("{}", e).as_str()),
        }
    }

    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        match self.call_with_args("get_manga_info", &path) {
            Ok(res) => res,
            Err(e) => ExtensionResult::err(format!("{}", e).as_str()),
        }
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        match self.call_with_args("get_chapters", &path) {
            Ok(res) => res,
            Err(e) => ExtensionResult::err(format!("{}", e).as_str()),
        }
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        match self.call_with_args("get_pages", &path) {
            Ok(res) => res,
            Err(e) => ExtensionResult::err(format!("{}", e).as_str()),
        }
    }
}

pub fn start() -> (JoinHandle<()>, Sender<Command>) {
    let (tx, rx) = tokio::sync::mpsc::channel(25);
    let handle = tokio::spawn(async move {
        thread_main(rx).await;
    });

    (handle, tx)
}

pub async fn load<P: AsRef<Path>>(
    path: P,
    tx: Sender<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    match tokio::fs::read_dir(&path).await {
        Ok(_) => {}
        Err(_) => {
            let _ = std::fs::create_dir_all(&path);
        }
    }

    #[cfg(feature = "compiler")]
    compile(&path).await?;

    let mut read_dir = tokio::fs::read_dir(&path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if !entry
            .path()
            .extension()
            .map_or(false, |ext| ext == "tanoshi")
        {
            continue;
        }

        let path = entry.path();
        info!("found compiled plugin at {:?}", path.clone());
        tx.send(Command::Load(
            path.to_str().ok_or("no path str")?.to_string(),
        ))
        .await?;
    }

    Ok(())
}

#[cfg(feature = "compiler")]
pub async fn compile<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    compile_with_target(path, env!("TARGET"), true).await?;

    Ok(())
}

#[cfg(feature = "compiler")]
pub async fn compile_with_target<P: AsRef<Path>>(
    path: P,
    triple: &str,
    remove_wasm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::str::FromStr;
    use wasmer::{CpuFeature, RuntimeError, Triple};

    info!("compile wasm for {}", triple);

    let triple = Triple::from_str(triple).map_err(|error| RuntimeError::new(error.to_string()))?;
    let cpu_feature = CpuFeature::set();
    let target = Target::new(triple, cpu_feature);

    match tokio::fs::read_dir(&path).await {
        Ok(_) => {}
        Err(_) => {
            let _ = std::fs::create_dir_all(&path);
        }
    }

    let mut read_dir = tokio::fs::read_dir(&path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if !entry.path().extension().map_or(false, |ext| ext == "wasm") {
            continue;
        }

        let path = entry.path();
        info!("found wasm file at {:?}", path.clone());
        ExtensionProxy::compile_from_file(&path, target.clone()).map_err(|e| format!("{}", e))?;

        if remove_wasm {
            debug!("remove wasm file");
            tokio::fs::remove_file(path).await?;
        }
    }

    Ok(())
}

async fn thread_main(extension_receiver: Receiver<Command>) {
    let mut recv = extension_receiver;
    let mut extension_map: BTreeMap<i64, (Source, Arc<dyn Extension>)> = BTreeMap::new();

    while let Some(cmd) = recv.recv().await {
        match cmd {
            Command::Insert(source_id, proxy) => {
                let source = proxy.detail();
                extension_map.insert(source_id, (source, proxy));
            }
            Command::Load(path) => {
                info!("load plugin from {:?}", path);
                let now = Instant::now();
                let proxy = ExtensionProxy::new(&path);
                let source = proxy.detail();
                info!("loaded in {} ms: {:?}", now.elapsed().as_millis(), source);
                extension_map.insert(source.id, (source, proxy));
            }
            Command::Unload(source_id, tx) => {
                drop(extension_map.remove(&source_id));

                if tx.send(()).is_err() {
                    error!("[Command::Unload] receiver dropped");
                }
            }
            Command::Exist(source_id, tx) => {
                let exist = extension_map.get(&source_id).is_some();
                if tx.send(exist).is_err() {
                    error!("[Command::Exist] receiver dropped");
                }
            }
            Command::List(tx) => {
                let sources = extension_map
                    .values()
                    .cloned()
                    .map(|(source, _)| source)
                    .collect::<Vec<Source>>();

                if tx.send(sources).is_err() {
                    error!("[Command::List] receiver dropped");
                }
            }
            Command::Detail(source_id, tx) => match extension_map.get(&source_id) {
                Some((detail, _)) => {
                    if tx.send(detail.clone()).is_err() {
                        error!("[Command::Detail] receiver dropped");
                    }
                }
                None => {
                    error!("extension with id {} not found", source_id);
                }
            },
            Command::Filters(source_id, tx) => {
                process(&extension_map, source_id, tx, |proxy| proxy.filters()).await;
            }
            Command::GetMangaList(source_id, param, tx) => {
                process(&extension_map, source_id, tx, |proxy| {
                    proxy.get_manga_list(param)
                })
                .await;
            }
            Command::GetMangaInfo(source_id, path, tx) => {
                process(&extension_map, source_id, tx, |proxy| {
                    proxy.get_manga_info(path)
                })
                .await;
            }
            Command::GetChapters(source_id, path, tx) => {
                process(&extension_map, source_id, tx, |proxy| {
                    proxy.get_chapters(path)
                })
                .await;
            }
            Command::GetPages(source_id, path, tx) => {
                process(&extension_map, source_id, tx, |proxy| proxy.get_pages(path)).await;
            }
        }
    }
}

async fn process<F, T>(
    extension_map: &BTreeMap<i64, (Source, Arc<dyn Extension>)>,
    source_id: i64,
    tx: ExtensionResultSender<T>,
    f: F,
) where
    F: FnOnce(Arc<dyn Extension>) -> ExtensionResult<T> + Send + 'static,
    T: Send + 'static,
{
    match extension_map.get(&source_id) {
        Some(proxy) => {
            let (_, proxy) = proxy.clone();
            tokio::task::spawn_blocking(move || {
                let res = f(proxy);
                if tx.send(res).is_err() {
                    error!("[process] receiver dropped");
                }
            })
            .await
            .expect("failed to spawn process");
        }
        None => {
            error!("extension with id {} not found", source_id);
        }
    }
}

fn wasi_read_err(env: &ExtensionEnv) -> Result<String, Box<dyn std::error::Error>> {
    let mut state = env.wasi_env.state();
    let wasm_stderr = state.fs.stderr_mut()?.as_mut().ok_or("no wasi sterr")?;
    let mut buf = String::new();
    wasm_stderr.read_to_string(&mut buf)?;
    Ok(buf)
}

fn wasi_read(env: &ExtensionEnv) -> Result<String, Box<dyn std::error::Error>> {
    let mut state = env.wasi_env.state();
    let wasm_stdout = state.fs.stdout_mut()?.as_mut().ok_or("no wasi stdout")?;
    let mut buf = String::new();
    wasm_stdout.read_to_string(&mut buf)?;
    Ok(buf)
}

fn wasi_write(
    env: &ExtensionEnv,
    param: &impl Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = env.wasi_env.state();
    let wasm_stdout = state.fs.stdin_mut()?.as_mut().ok_or("no wasi stdin")?;

    let buf = ron::to_string(param)?;
    wasm_stdout.write_all(buf.as_bytes())?;

    Ok(())
}

fn host_http_request(env: &ExtensionEnv) {
    let http_req_str = match wasi_read(env) {
        Ok(http_req_str) => http_req_str,
        Err(e) => {
            error!("error wasi_read: {}", e);
            return;
        }
    };

    let http_req = match ron::from_str::<Request>(&http_req_str) {
        Ok(http_req) => http_req,
        Err(e) => {
            error!("error parsing http_req: {}", e);
            return;
        }
    };

    let http_res = tanoshi_util::http::http_request(http_req);

    match wasi_write(env, &http_res) {
        Ok(_) => {}
        Err(e) => {
            error!("error wasi_write: {}", e);
        }
    }
}

fn host_debug(env: &ExtensionEnv) {
    let message = match wasi_read_err(env) {
        Ok(message) => message,
        Err(e) => {
            error!("error wasi_read: {}", e);
            return;
        }
    };

    tanoshi_util::log::print_debug(message);
}

fn host_error(env: &ExtensionEnv) {
    let message = match wasi_read_err(env) {
        Ok(message) => message,
        Err(e) => {
            error!("error wasi_read: {}", e);
            return;
        }
    };

    tanoshi_util::log::print_error(message);
}

fn host_info(env: &ExtensionEnv) {
    let message = match wasi_read_err(env) {
        Ok(message) => message,
        Err(e) => {
            error!("error wasi_read: {}", e);
            return;
        }
    };

    tanoshi_util::log::print_info(message);
}

fn host_trace(env: &ExtensionEnv) {
    let message = match wasi_read_err(env) {
        Ok(message) => message,
        Err(e) => {
            error!("error wasi_read: {}", e);
            return;
        }
    };

    tanoshi_util::log::print_trace(message);
}

fn host_warn(env: &ExtensionEnv) {
    let message = match wasi_read_err(env) {
        Ok(message) => message,
        Err(e) => {
            error!("error wasi_read: {}", e);
            return;
        }
    };

    tanoshi_util::log::print_warn(message);
}
