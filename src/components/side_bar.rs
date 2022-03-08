use super::category::CategoriesComponent;
use super::state::{SideBarMessage, Events, State};
use crate::services::bookmark_api::BookmarkApi;
use iced::{
    Column, Element, Clipboard, Command
};

#[derive(Debug, Clone)]
pub struct SideBar {
    categories_component: CategoriesComponent,
}

impl SideBar {
    pub fn new() -> Self {
        let mut bookmarks_api = BookmarkApi::init();
        SideBar {
            categories_component: CategoriesComponent::new(
                &mut bookmarks_api.get_categories().to_vec(),
            ),
        }
    }
    pub fn update(
        &mut self,
        message: SideBarMessage,
        clipboard: &mut Clipboard,
        state: &mut State
    ) -> Command<Events> {
            match message {
                SideBarMessage::CategoryMessage(category_message) => self.categories_component.update(category_message, clipboard, state)
            }
    }
    pub fn view(&mut self) -> Element<SideBarMessage> {
        Column::new()
            .push(
                self.categories_component
                    .view()
                    .map(|ms| SideBarMessage::CategoryMessage(ms)),
            )
            .into()
    }
}
