use iced::{button, Align, Button, Container, Element, Length, Row, Text, scrollable, Scrollable, VerticalAlignment, HorizontalAlignment, Rule, Column, Space, Clipboard, Command};
// use futures::Future;

use crate::{style::ButtonStylingType, services::{bookmark_api::{BookmarkCategory, BookmarkApi}}};

use super::state::{CategoryMessage, Events, State};
use crate::style;

#[derive(Clone, Debug)]
struct CategoryComponent {
    button_state: button::State,
    data: BookmarkCategory, // bookmark_api: BookmarkApi
}

impl CategoryComponent {
    pub fn new(data: BookmarkCategory) -> Self {
        CategoryComponent {
            button_state: button::State::new(),
            data,
        }
    }
    pub fn view(&mut self, index: usize) -> Element<CategoryMessage> {
        let container: Container<CategoryMessage> = Container::new(
            // Text::new(&self.data.name),
    Button::new(&mut self.button_state, Text::new(&self.data.name)
            .vertical_alignment(VerticalAlignment::Center)
            .width(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Center)
        ).padding(10).width(Length::Fill)
        .style(style::StyledButton::new(ButtonStylingType::Index(index)))
        .on_press(CategoryMessage::CategoryClicked(self.data.to_owned()))
        );
        container
        .width(Length::FillPortion(1))
        .center_y()
        .into()
    }
}

#[derive(Clone, Debug)]
pub struct CategoriesComponent {
    scroll_state: scrollable::State,
    categories: Vec<CategoryComponent>,
}

impl CategoriesComponent {
    pub fn new(categories: &mut Vec<BookmarkCategory>) -> Self {
        CategoriesComponent {
            scroll_state: scrollable::State::new(),
            categories: Vec::from_iter(
                categories
                    .iter()
                    .map(|item| CategoryComponent::new(item.clone())),
            ),
        }
    }
    pub fn update
    (
        &mut self,
        message: CategoryMessage,
        _clipboard: &mut Clipboard,
        state: &mut State
    )
    -> Command<Events>
     {
        match message {
            CategoryMessage::CategoryClicked(cat) => {
                *state = State::LoadItems(0, Some(cat), None);
                Command::none()
            },
            CategoryMessage::Reload(_) => {
                let mut categories: Vec<BookmarkCategory> = {
                    let mut bookmark_api = BookmarkApi::init();
                    bookmark_api.get_categories().to_vec()
                };
                *self = Self::new(&mut categories);
                Command::none()
            }
        }
    }
    pub fn view(&mut self) -> Element<CategoryMessage> {
        let mut all_content: Scrollable<CategoryMessage> = Scrollable::new(&mut self.scroll_state)
        .align_items(Align::Center)
        .height(Length::Units(450))
        .padding(15);
        all_content = all_content.push(
            Column::new()
            .width(Length::Fill)
            .align_items(Align::Center)
            .push(Space::new(Length::Fill, Length::Units(20)))
            .push(
                Text::new("Select A Bookmark Category").size(18).color(style::TEXT_COLOR)
            )
            .push(Rule::horizontal(30))
        );
        let mut row: Row<CategoryMessage> = Row::new()
            .width(Length::Fill)
            .spacing(5)
            .padding(5)
            .align_items(Align::Center);
        let mut splicer = 0;
        let mut index = 0;
        let cat_length = self.categories.len();
        let columns_count: u16 = 2;
        for component in self.categories.iter_mut() {
            index += 1;
            row = row.push(component.view(index)).align_items(Align::Center);
            if splicer == columns_count - 1 {
                all_content = all_content.push(row);
                row = Row::new()
                    .width(Length::Fill)
                    .spacing(10)
                    .padding(5)
                    .align_items(Align::Center);
                splicer = 0;
            } else if index == cat_length {
                all_content = all_content.push(row);
                row = Row::new()
                    .width(Length::Fill)
                    .spacing(10)
                    .padding(5)
                    .align_items(Align::Center);
                splicer = 0;
            } else {
                splicer += 1;
            }
        }
        all_content.into()
    }
}

