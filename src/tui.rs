use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListState, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};
use readability_rust::Readability;
use tokio::io;
use tui_markdown;

use crate::config::Config;
use crate::rss_def::OxideArticle;

const OXIDE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum WinState {
    Feeds,
    Articles,
    Content,
}

pub struct App {
    config: Config,
    articles: Vec<OxideArticle>,
    article_state: ListState,
    article_content: Option<String>,
    feed_state: ListState,
    content_scroll: u16,
    win_state: WinState,
    exit: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut article_state = ListState::default();
        article_state.select(Some(0));
        let mut feed_state = ListState::default();
        feed_state.select(Some(0));
        Self {
            config,
            articles: Vec::<OxideArticle>::new(),
            article_state,
            article_content: None,
            feed_state,
            win_state: WinState::Feeds,
            content_scroll: 0,
            exit: false,
        }
    }
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).await;
            }
            _ => {}
        };
        Ok(())
    }

    async fn handle_key_event(&mut self, mut key_event: KeyEvent) {
        if let KeyCode::Char(c) = key_event.code {
            if c == self.config.down.unwrap_or('j') {
                self.arrow_down();
                return;
            }
            if c == self.config.up.unwrap_or('k') {
                self.arrow_up();
                return;
            }
            if c == self.config.select.unwrap_or('l') {
                key_event.code = KeyCode::Enter;
            }
            if c == self.config.back.unwrap_or('h') {
                key_event.code = KeyCode::Esc;
            }
        }
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down => self.arrow_down(),
            KeyCode::Up => self.arrow_up(),
            KeyCode::Enter => {
                match self.win_state {
                    WinState::Feeds => {
                        self.win_state = WinState::Articles;
                        self.articles = match self.get_articles().await {
                            Ok(x) => x,
                            Err(_) => Vec::<OxideArticle>::new(),
                        };
                        self.article_state.select(Some(0));
                    }
                    WinState::Articles => {
                        self.win_state = WinState::Content;
                        if self.articles.len() == 0 {
                            return;
                        }
                        self.articles[self.article_state.selected().unwrap_or(0)].is_read = true;
                        if let Some(index) = self.article_state.selected() {
                            if let Some(article) = self.articles.get(index) {
                                let md_content =
                                    html2text::from_read(article.content.as_bytes(), 1145)
                                        .unwrap_or_else(|_| {
                                            "Failed to parse the article".to_string()
                                        });
                                self.article_content = Some(md_content);
                            }
                        }
                    }
                    WinState::Content => {
                        if self.articles.len() == 0 {
                            return;
                        }
                        let link = self.articles[self.article_state.selected().unwrap_or(0)]
                            .link
                            .clone();
                        let new_content = self.get_html_link(link).await;
                        match new_content {
                            Ok(x) => {
                                let md_content = html2text::from_read(x.as_bytes(), 1145)
                                    .unwrap_or_else(|_| "Failed to parse the article".to_string());
                                self.article_content = Some(md_content);
                            }
                            Err(_) => return,
                        }
                        self.win_state = WinState::Content;
                    }
                };
            }
            KeyCode::Esc => {
                match self.win_state {
                    WinState::Content => {
                        self.win_state = WinState::Articles;
                        self.article_content = Some("".to_string());
                        self.content_scroll = 0;
                    }
                    WinState::Articles => {
                        self.win_state = WinState::Feeds;
                        self.article_state.select(Some(0));
                        self.articles = Vec::<OxideArticle>::new();
                    }
                    WinState::Feeds => self.win_state = WinState::Feeds,
                };
            }
            _ => {}
        }
    }

    fn next_article(&mut self) {
        let i = match self.article_state.selected() {
            Some(i) => {
                if i >= self.articles.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.article_state.select(Some(i));
    }

    fn prev_article(&mut self) {
        let i = match self.article_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.articles.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.article_state.select(Some(i));
    }

    fn next_feed(&mut self) {
        let i = match self.feed_state.selected() {
            Some(i) => {
                if i >= self.config.feeds.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.feed_state.select(Some(i));
    }

    fn prev_feed(&mut self) {
        let i = match self.feed_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.config.feeds.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.feed_state.select(Some(i));
    }

    fn scroll_down(&mut self) {
        self.content_scroll += 1;
    }

    fn scroll_up(&mut self) {
        if self.content_scroll != 0 {
            self.content_scroll -= 1;
        }
    }

    fn arrow_down(&mut self) {
        match self.win_state {
            WinState::Feeds => self.next_feed(),
            WinState::Articles => self.next_article(),
            WinState::Content => self.scroll_down(),
        }
    }

    fn arrow_up(&mut self) {
        match self.win_state {
            WinState::Feeds => self.prev_feed(),
            WinState::Articles => self.prev_article(),
            WinState::Content => self.scroll_up(),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    async fn get_articles(&mut self) -> Result<Vec<OxideArticle>, Box<dyn std::error::Error>> {
        let index = self.feed_state.selected().unwrap_or(0);
        let url = &self.config.feeds[index].link;
        let response = reqwest::get(url).await?.bytes().await?;

        let channel = rss::Channel::read_from(&response[..])?;
        let article: Vec<OxideArticle> = channel
            .items()
            .iter()
            .cloned()
            .map(OxideArticle::from)
            .collect();
        Ok(article)
    }

    async fn get_html_link(&mut self, link: String) -> Result<String, Box<dyn std::error::Error>> {
        let html = reqwest::get(link).await?.text().await?;
        let mut parser = Readability::new(html.as_str(), None)?;
        let res = match parser.parse() {
            Some(x) => x.content.unwrap_or("Failed to read article".to_string()),
            None => "Failed to read article".to_string(),
        };
        Ok(res)
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let oxide_title = Line::from(" Oxide RSS Reader ".to_string().bold().red());
        let oxide_version = Line::from(format!(" {} ", OXIDE_VERSION).to_string().bold().red());
        let outer_block = Block::bordered()
            .title(oxide_title)
            .title_bottom(oxide_version.right_aligned());
        let inner_area = outer_block.inner(area);
        outer_block.render(area, buf);

        let [feed_area, article_area, content_area] = match self.win_state {
            WinState::Feeds => Layout::horizontal([
                Constraint::Percentage(90),
                Constraint::Percentage(5),
                Constraint::Percentage(5),
            ])
            .areas(inner_area),
            WinState::Articles => Layout::horizontal([
                Constraint::Percentage(5),
                Constraint::Percentage(90),
                Constraint::Percentage(5),
            ])
            .areas(inner_area),
            WinState::Content => Layout::horizontal([
                Constraint::Percentage(5),
                Constraint::Percentage(5),
                Constraint::Percentage(90),
            ])
            .areas(inner_area),
        };

        let feeds_items: Vec<String> = self.config.feeds.iter().map(|a| a.name.clone()).collect();
        let articles_items: Vec<Line> = self
            .articles
            .iter()
            .map(|a| {
                if !a.is_read {
                    Line::from(format!("{} - {}", a.published_at.clone(), a.title.clone())).bold()
                } else {
                    Line::from(format!("{} - {}", a.published_at.clone(), a.title.clone())).italic()
                }
            })
            .collect();

        let spacer = Block::default()
            .borders(Borders::RIGHT)
            .padding(Padding::horizontal(1));

        let feeds_list = List::new(feeds_items)
            .block(spacer.clone())
            .highlight_style(Style::default().red().bold());

        let articles_list = List::new(articles_items)
            .block(spacer)
            .highlight_style(Style::default().red().bold());

        StatefulWidget::render(feeds_list, feed_area, buf, &mut self.feed_state);
        StatefulWidget::render(articles_list, article_area, buf, &mut self.article_state);

        if let Some(text) = &self.article_content {
            Paragraph::new(tui_markdown::from_str(text.as_str()))
                .block(
                    Block::default()
                        .padding(Padding::horizontal(1))
                        .border_type(ratatui::widgets::BorderType::Rounded),
                )
                .wrap(Wrap { trim: true })
                .scroll((self.content_scroll, 0))
                .render(content_area, buf);
        }
    }
}
