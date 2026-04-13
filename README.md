# Oxide RSS Reader

Oxide is an RSS Reader made in Rust. It's pretty simple and will not do much but it's currently enough for me.

## How to use

You'll need to create a configuration file `~/.config/oxide/oxide.toml` with a feeds list.

```toml
# Optionnal
back = 'j'
down_key = 'k'
up_key = 'l'
select = 'm'

# Required
feeds = [
	{name = "General RSS", link = "https://www.rssweblog.com/rss-feed"},
	{name = "This week in Rust", link = "https://this-week-in-rust.org/rss.xml"},
	{name = "HackerNews Front Page", link = "https://hnrss.org/frontpage"},
	{name = "Journal du Hacker", link = "https://www.journalduhacker.net/rss"},
]

```

## Future of Oxide

I might go all-in and create some kind of "database-based" app. For now I'm fine with this method.

## Libraries used

- `reqwest`: to get the content of the feed
- `tokio`: to make stuff async
- `toml` and `serde`: to parse the config file
- `rss`: to parse RSS stuff
- `html2text`: to create markdown from the HTML page
- `ratatui`: to create the TUI
- `tui-markdown`: to make the content more readable
