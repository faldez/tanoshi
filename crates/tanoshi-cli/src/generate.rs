use crate::data::Source;
use tanoshi_vm::bus::ExtensionBus;

pub async fn generate_json(bus: ExtensionBus) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("repo");
    let _ = std::fs::create_dir_all(path.join("library"));

    let sources = bus
        .list()
        .await?
        .iter()
        .map(|detail| Source {
            id: detail.id,
            name: detail.name.clone(),
            path: format!("library/{}.wasm", detail.name),
            version: detail.version.to_string(),
            icon: detail.icon.clone(),
        })
        .collect::<Vec<Source>>();

    let file = std::fs::File::create(path.join("index.json"))?;
    serde_json::to_writer(&file, &sources)?;

    Ok(())
}
