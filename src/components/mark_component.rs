// use rust_embed::RustEmbed;

// #[derive(RustEmbed)]
// #[folder = "assets/"]
// #[prefix = "assets/"]
// pub struct Asset;


use iced::{
    Align, Clipboard, Column, Command, Container, Element, Image, Length, Row, Text, Rule, HorizontalAlignment, Space, image, Button, button, TextInput, text_input
};
use super::state::{MCMessage, State, Events};
use crate::{style, services::{ bookmark_api::{MarkData, BookmarkApi, BookmarkCategory}, asset::Asset}, config, utils};
use open;

#[derive(Clone, Debug)]
pub struct MarkComponent {
    pub data: MarkData,
    button_state: button::State,
}

impl MarkComponent {
    pub fn new(data: MarkData) -> Self {
        MarkComponent { data, button_state: button::State::new() }
    }
    pub fn view(&mut self) -> Element<MCMessage> {
        let image_handler: image::Handle = match &self.data.image.as_str() {
            &config::DEFAULT_IMG_PATH => {
                let img_path = config::get_default_image_path();
                let re = Asset::get(img_path.to_str().unwrap()).unwrap().data;
                image::Handle::from_memory(re.to_vec())
            },
            _ => {
                let split_path: Vec<&str> = self.data.image.split("?").collect();
                let img_path = config::get_full_img_cache_path().join(split_path[0]);
                image::Handle::from_path(img_path)
            }
        };
        let max_chars: usize = 60;
        let title = utils::truncate_with_dots(&mut self.data.title, max_chars);
        Container::new(
            Column::new()
                .push(
                    Column::new()
                    .push(
                        Image::new(
                            image_handler
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                    )
                    .width(Length::Fill)
                    .height(Length::Units(210))
                    .align_items(Align::Center)
                )
                .push(Space::new(Length::Fill, Length::Units(20)))
                .push(
                    Column::new()
                    .push(Text::new(title))
                    // .push(Text::new(&self.data.description))
                    .align_items(Align::Center)
                )
                .push(
                    Column::new()
                    .push(
                    Button::new(
                            &mut self.button_state,
                    Text::new("Go to")
                            .width(Length::Fill)
                            .horizontal_alignment(HorizontalAlignment::Center)
                        )
                        .width(Length::Units(100))
                        .padding(10)
                        .style(style::StyledButton::accent())
                        .on_press(MCMessage::GotoClicked(self.data.clone()))
                    )
                    .push(Space::new(Length::Fill, Length::Units(20)))
                    .push(
                        Text::new(&self.data.category.name)
                        .color(style::SECONDARY_COLOR)
                )
                    .align_items(Align::End)
                    .padding(30)
                )
                .height(Length::Fill)
                .align_items(Align::Center)
                .padding(10), // .width(Length::FillPortion(1))
        )
        .padding(5)
        .align_y(Align::Center)
        .style(style::FeedCard)
        .width(Length::Fill)
        .height(Length::Units(460))
        .into()
    }
}

#[derive(Clone, Debug)]
pub struct MarkComponents {
    items: Vec<MarkComponent>,
    current_cat_id: Option<String>,
    load_more_btn_state: button::State,
    search_input_state: text_input::State,
    search_input_val: String,
    last_index: usize,
    has_more: bool
}

impl MarkComponents {
    pub fn new() -> Self {
        let all_items: Vec<MarkData> = Vec::new();
        MarkComponents {
            items: Self::render_items(&all_items),
            current_cat_id: None,
            load_more_btn_state: button::State::new(),
            search_input_state: text_input::State::new(),
            search_input_val: String::new(),
            last_index: 0,
            has_more: false
        }
    }
    fn render_items(marks: &Vec<MarkData>) -> Vec<MarkComponent> {
        Vec::from_iter(
            marks
                .iter()
                .map(|item| MarkComponent::new(item.clone())),
        )
    }
    pub fn update(
        &mut self,
        message: &MCMessage,
        _clipboard: &mut Clipboard,
        state: &mut State
    ) -> Command<Events> {
        match message{
            &MCMessage::Refresh(_) => {
                self.items = Vec::new();
                *state = State::None;
            }
            MCMessage::LoadMark(mark_meta) => {
                match &mark_meta.mark {
                    Some(mark) => {
                        if self.search_input_val.is_empty() {
                            match &self.current_cat_id {
                                Some(current_cat_id) => {
                                    if current_cat_id != &mark.category.id {
                                        self.items = Vec::new();
                                        self.current_cat_id = Some(mark.category.id.to_string());
                                        self.search_input_val = String::new();
                                    }
                                },
                                None => {
                                    self.current_cat_id = Some(mark.category.id.to_string());
                                },
                            };
                        }
                        if mark_meta.index == 0 {
                            self.items = Vec::new();
                        }
                        self.items.push(MarkComponent::new(mark.clone()));
                        self.last_index = mark_meta.index;
                    },
                    None => {
                        self.items = Vec::new();
                        self.last_index = 0;
                        self.current_cat_id = None;
                        *state = State::None;
                    }
                }
                self.has_more = mark_meta.has_more;
                *state = State::None;
            },
            MCMessage::GotoClicked(mark) => {
                open::that(&mark.link).ok();
            },
            MCMessage::LoadMore(i) => {
                let index = &(i + 1);
                if self.current_cat_id.is_some() && self.search_input_val.is_empty() {
                    let mut bookmark_api = BookmarkApi::init();
                    let cat = bookmark_api.get_category(&self.current_cat_id.as_ref().unwrap());
                    if cat.is_some() {
                        *state = State::LoadItems(*index, Some(cat.unwrap().clone()), None);
                    } else {
                        *state = State::None;
                    }
                } else if self.current_cat_id.is_none() && !self.search_input_val.is_empty() {
                    *state = State::LoadItems(*index, None, Some(self.search_input_val.to_string()));
                } else {
                    *state = State::None;
                }
            },
            MCMessage::SearchInputChanged(val) => {
                self.search_input_val = val.to_string();
            },
            MCMessage::Search => {
                self.last_index = 0;
                self.current_cat_id = None;
                if !self.search_input_val.is_empty() {
                    self.items = Vec::new();
                    *state = State::LoadItems(self.last_index, None, Some(self.search_input_val.to_string()));
                } else {
                    *state = State::LoadItems(self.last_index, Some(BookmarkCategory::default()), None);
                }
            },
            MCMessage::CategoryClicked => {
                self.current_cat_id = None;
                self.search_input_val = String::new();
            }
        }
        Command::none()
    }
    fn render_search_input<'a>(state: &'a mut text_input::State, search_input_val: &'a String) -> TextInput<'a, MCMessage> {
        TextInput::new(state, "search in bookmarks", search_input_val, MCMessage::SearchInputChanged)
        .padding(15).width(Length::Fill).size(20)
        .style(style::StyledTextInput::new())
        .on_submit(MCMessage::Search)
    }
    fn render_content(&mut self) -> Element<MCMessage> {
        let items_length = &self.items.len().clone();
        let mut all_content = Column::new().push(Space::new(Length::Fill, Length::Units(20)));
        all_content = all_content.push(
            Self::render_search_input(&mut self.search_input_state, &self.search_input_val)
        )
        .push(Space::new(Length::Fill, Length::Units(20)))
        ;
        if items_length == &0 && self.current_cat_id.is_none() {
            all_content = all_content.push(
                Text::new("no data to show").width(Length::Fill).size(20)
                .color(style::TEXT_COLOR)
                .horizontal_alignment(HorizontalAlignment::Center)
            )
            .push(Rule::horizontal(10))
        }
        let mut row: Row<MCMessage> = Row::new()
            .width(Length::Fill)
            .spacing(50)
            .padding(20)
            .align_items(Align::Center);
        let mut splicer = 0;
        let mut count = 0;
        let marks_count = self.items.len();
        let columns_count: u16 = 2;
        for item in self.items.iter_mut() {
            count += 1;
            row = row.push(item.view());
            if splicer == columns_count - 1 {
                all_content = all_content.push(row);
                row = Row::new()
                    .width(Length::Fill)
                    .spacing(50)
                    .padding(20)
                    .align_items(Align::Center);
                splicer = 0;
            } else if count == marks_count {
                all_content = all_content.push(row);
                row = Row::new()
                    .width(Length::Fill)
                    .spacing(50)
                    .padding(20)
                    .align_items(Align::Center);
                splicer = 0;
            } else {
                splicer += 1;
            }
        }
        if items_length > &0 && self.has_more {
            all_content = all_content.push(
                Column::new()
                    .push(
                        Button::new(&mut self.load_more_btn_state, Text::new("load more"))
                        .padding(10)
                        .style(style::StyledButton::primary())
                        .on_press(MCMessage::LoadMore(self.last_index))
                    )
                    .push(Space::new(Length::Fill, Length::Units(30)))
                    .align_items(Align::Center)
                );
        }
        Container::new(all_content)
            .align_x(Align::Center)
            .align_y(Align::Center)
            .padding(30)
            .into()
    }
    pub fn view(&mut self) -> Element<MCMessage> {
        self.render_content()
    }
}
