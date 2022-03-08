use crate::{services::bookmark_api::{MarkData, BookmarkCategory, MarkMeta}, utils::Error};

#[derive(Debug, Clone)]
pub enum Events{
}


#[derive(Clone, Debug)]
pub enum MCMessage {
    LoadMark(MarkMeta),
    GotoClicked(MarkData),
    LoadMore(usize),
    Refresh(Result<(), Error>),
    SearchInputChanged(String),
    Search,
    CategoryClicked
}

#[derive(Clone, Debug)]
pub enum CategoryMessage {
    CategoryClicked(BookmarkCategory),
    Reload(Result<(), Error>)
}

#[derive(Clone, Debug)]
pub enum SideBarMessage {
    CategoryMessage(CategoryMessage),
}
#[derive(Clone, Debug)]
pub enum HeaderMessage{
    Loading,
    Loaded,
    ClearCache,
    Resync
}

#[derive(Clone, Debug)]
pub enum Message {
    MCEvent(MCMessage),
    SideBarMessage(SideBarMessage),
    HeaderMessage(HeaderMessage),
    Events(Events),
    Syncing(Result<(), Error>),
    Synced(())
}

#[derive(Clone, Debug)]
pub enum State{
    LoadItems(usize, Option<BookmarkCategory>, Option<String>),
    None
}
