pub struct OxideArticle {
    // pub guid: String,
    pub title: String,
    pub link: String,
    pub content: String,
    // pub author: String,
    pub published_at: String,
    pub is_read: bool,
}

impl From<rss::Item> for OxideArticle {
    fn from(item: rss::Item) -> Self {
        // let guid = item
        //     .guid()
        //     .map(|x| x.value().to_string())
        //     .unwrap_or_else(|| item.link().unwrap_or("no-id").to_string());
        let title = item.title().unwrap_or("No Title").to_string();
        let link = item.link().unwrap_or("").to_string();
        let content = item
            .extensions()
            .get("content")
            .and_then(|ext| ext.get("encoded"))
            .and_then(|v| v.first())
            .and_then(|ev| ev.value())
            .or(item.description())
            .unwrap_or("")
            .to_string();
        // let author = item
        //     .author()
        //     .or_else(|| {
        //         item.dublin_core_ext()
        //             .and_then(|dc| dc.creators().first().map(|s| s.as_str()))
        //     })
        //     .unwrap_or("Unknown Author")
        //     .to_string();
        let published_at = item
            .pub_date()
            .unwrap_or("Unknown date")
            .to_string()
            .split_at(16)
            .0
            .split_at(5)
            .1
            .to_string();
        OxideArticle {
            // guid,
            title,
            link,
            content,
            // author,
            published_at,
            is_read: false,
        }
    }
}
