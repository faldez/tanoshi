use tanoshi_lib::prelude::Param;
use tanoshi_vm::bus::ExtensionBus;

pub async fn test(
    bus: ExtensionBus,
    selector: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for detail in bus.list().await? {
        if let Some(selector) = selector.clone() {
            if detail.name != selector {
                continue;
            }
        }

        println!("Test {}", detail.name);

        print!("Test get supported filters ");
        let _ = bus.filters(detail.id).await?;
        println!("ok");

        let param = Param::default();

        print!("Test get_manga_list ");
        let manga = bus.get_manga_list(detail.id, param).await?;
        println!("ok");

        print!("Test get_manga_info {} ", manga[0].path.clone());
        let _ = bus.get_manga_info(detail.id, manga[0].path.clone()).await?;
        println!("ok");

        print!("Test get_chapters {} ", manga[0].path.clone());
        let chapters = bus.get_chapters(detail.id, manga[0].path.clone()).await?;
        println!("ok");

        print!("Test get_pages {} ", chapters[0].path.clone());
        let _ = bus.get_pages(detail.id, chapters[0].path.clone()).await?;
        println!("ok");
    }

    Ok(())
}
