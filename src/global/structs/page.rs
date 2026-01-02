use std::fmt::Debug;

use crate::config::*;
use tui_additions::framework::Framework;
use typemap::Key;

// Page can be converted into PageConfig, which can then be converted into State
/// Covers all possible pages and variants
#[derive(Clone, PartialEq, Eq)]
pub enum Page {
    MainMenu(MainMenuPage),
    // Option<new channel selected index>
    Feed,
    Search(Search),
    SingleItem(SingleItemPage),
    ChannelDisplay(ChannelDisplayPage),
    Login(LoginPage),
}

impl Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}(_)",
            match self {
                Self::MainMenu(_) => "MainMenu",
                Self::Feed => "Feed",
                Self::Search(_) => "Search",
                Self::SingleItem(_) => "SingleItem",
                Self::ChannelDisplay(_) => "ChannelDisplay",
                Self::Login(_) => "Login",
            }
        ))
    }
}

impl Page {
    pub fn channeldisplay(&self) -> &ChannelDisplayPage {
        if let Self::ChannelDisplay(channeldisplaypage) = self {
            channeldisplaypage
        } else {
            panic!("not a channel display");
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::MainMenu(MainMenuPage::default())
    }
}

impl Key for Page {
    type Value = Self;
}

/// page variants for the main menu
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum MainMenuPage {
    #[default]
    Trending,
    Popular,
    History,
    Library,
}

/// variants of the coannel display page
#[derive(Clone, PartialEq, Eq)]
pub struct ChannelDisplayPage {
    pub id: String,
    pub r#type: ChannelDisplayPageType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChannelDisplayPageType {
    Main,
    Videos,
    Playlists,
}

/// Different items to be displayed on a single item page
#[derive(Clone, PartialEq, Eq)]
pub enum SingleItemPage {
    Video(String),
    Playlist(String),
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct LoginPage {
    pub user_code: String,
    pub verification_url: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl Page {
    pub fn to_page_config(&self, framework: &Framework) -> PageConfig {
        let pages_config = framework.data.global.get::<PagesConfig>().unwrap();
        match self {
            Self::MainMenu(_) => pages_config.main_menu.clone(),
            Self::Feed => pages_config.feed.clone(),
            Self::Search(_) => pages_config.search.clone(),
            Self::SingleItem(_) => pages_config.singleitem.clone(),
            Self::ChannelDisplay(_) => pages_config.channeldisplay.clone(),
            Self::Login(_) => pages_config.login.clone(),
        }
    }

    // each page displays a text when loading, and that text is taken from config
    pub fn load_msg(&self, framework: &Framework) -> String {
        let pages_config = framework.data.global.get::<PagesConfig>().unwrap();
        match self {
            Self::MainMenu(_) => pages_config.main_menu.message.clone(),
            Self::Feed => pages_config.feed.message.clone(),
            Self::Search(_) => pages_config.search.message.clone(),
            Self::SingleItem(_) => pages_config.singleitem.message.clone(),
            Self::ChannelDisplay(_) => pages_config.channeldisplay.message.clone(),
            Self::Login(_) => String::from("Connecting to Google..."),
        }
    }
}
