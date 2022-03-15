
use super::state::{State, HeaderMessage, Message, CategoryMessage, SideBarMessage, MCMessage};
use iced::{Button, button};
use iced::{
    Column, Element, Clipboard, Command, Text, Container, Length, Row, VerticalAlignment, Svg, svg, Align
};

use crate::services::asset::Asset;
use crate::services::bookmark_api::{BookmarkApi};
use crate::style;
use crate::config;
#[derive(Debug, Clone)]
pub struct Header {
    loading: bool,
    clear_cache_btn_state: button::State,
    resync_btn_state: button::State,
    icon_handle: svg::Handle
}

impl Header {
    pub fn new() -> Self {
        let asset_handler = {
            let asset = Asset::get(config::get_loader_icon_path().to_str().unwrap());
            if asset.is_some() {
                svg::Handle::from_memory(asset.unwrap().data.to_vec())
            } else {
                svg::Handle::from_path(config::get_loader_icon_path().to_str().unwrap())
            }
            // println!("+=================== asset is {:#?}", asset);
            // if asset.is_som
        };
        Header {
            loading: false,
            clear_cache_btn_state: button::State::new(),
            resync_btn_state: button::State::new(),
            icon_handle: asset_handler
            //  svg::Handle::from_memory(Asset::get(config::get_loader_icon_path().to_str().unwrap()).unwrap().data.to_vec())
        }
    }
    pub fn update(
        &mut self,
        message: HeaderMessage,
        _clipboard: &mut Clipboard,
        _state: &mut State
    ) -> Command<Message> {
            match message{
                HeaderMessage::Loading => {
                    self.loading = true;
                    Command::batch([
                        Command::perform(BookmarkApi::sync_all(), Message::Synced),
                        Command::perform(BookmarkApi::perform_load(), CategoryMessage::Reload)
                        .map(|m| Message::SideBarMessage(SideBarMessage::CategoryMessage(m))),
                        Command::perform(BookmarkApi::perform_load(), MCMessage::Refresh)
                        .map(|m| Message::MCEvent(m))
                    ])
                },
                HeaderMessage::Loaded => {
                    self.loading = false;
                    Command::none()
                },
                HeaderMessage::ClearCache => {
                    Command::perform(BookmarkApi::flush_all_resync(), Message::Syncing)
                },
                HeaderMessage::Resync => {
                    self.loading = true;
                    Command::perform(BookmarkApi::perform_load(), Message::Syncing)
                }
            }
    }
    pub fn view(&mut self) -> Element<HeaderMessage> {
        let mut content: Row<HeaderMessage> = Row::new().height(Length::Units(60));
        content = content.push(
            Column::new().push(
                Text::new("Aan").color(style::TEXT_COLOR).size(25).height(Length::Fill).vertical_alignment(VerticalAlignment::Center)
            )
            .width(Length::FillPortion(2)).height(Length::Fill)
            // .padding(20)
        )
        .padding(10)
        .width(Length::Fill);
        if self.loading {
            content = content.push(
                Text::new("syncing and loading marks...")
                .color(style::TEXT_COLOR).width(Length::FillPortion(1))
                .vertical_alignment(VerticalAlignment::Center).height(Length::Fill)
            )
            .push(
                Svg::new(
                    self.icon_handle.clone()
                ).width(Length::Units(20)).height(Length::Units(20))
            )
            .align_items(Align::Center)
        }
         else {
            content = content.push(
            Column::new()
                .push(
                    Row::new()
                    .push(
                        Button::new(&mut self.clear_cache_btn_state, Text::new("clear cache & resync"))
                        .style(style::StyledButton::primary())
                        .padding(10)
                        .on_press(HeaderMessage::ClearCache)
                    )
                    .push(
                        Button::new(&mut self.resync_btn_state, Text::new("resync bookmarks"))
                        .style(style::StyledButton::accent())
                        .padding(10)
                        .on_press(HeaderMessage::Resync)
                    )
                    .width(Length::Fill)
                    .spacing(10)
                )
                .width(Length::FillPortion(1))
            );
            // .align_items(A)
        }
        Container::new(content)
            .width(Length::Fill)
            .style(style::Header)
            // .push()
            .into()
    }
}
