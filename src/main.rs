pub mod style;
pub mod services;
pub mod utils;
mod components;
pub mod subscriptions;
pub mod config;
use iced::{
    executor, scrollable, Application, Clipboard, Column, Command, Container, Element, Length, Row,
    Scrollable, Settings, Space, Subscription
};
use components::{mark_component::MarkComponents, side_bar::SideBar, state::{Message, HeaderMessage, CategoryMessage, SideBarMessage}, state::State, state::MCMessage, header::Header};
use services::bookmark_api::{BookmarkApi, BookmarkCategory};

#[tokio::main]
pub async fn main() -> iced::Result {
    let mut config = Settings::default();
    config.window.max_size = Some((1200, 900));
    config.window.min_size = Some((1200, 900));
    Mark::run(config)
}

struct Mark {
    body_scroll: scrollable::State,
    sidebar_scroll: scrollable::State,
    header: Header,
    mark_components: MarkComponents,
    side_bar: SideBar,
    state: State
}

impl Application for Mark {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    fn new(_flags: ()) -> (Mark, Command<Message>) {
        let app: Mark = Mark {
            body_scroll: scrollable::State::new(),
            sidebar_scroll: scrollable::State::new(),
            mark_components: MarkComponents::new(),
            header: Header::new(),
            side_bar: SideBar::new(),
            state: State::None
        };
        (app, Command::perform(BookmarkApi::perform_load(), Message::Syncing))
    }
    fn title(&self) -> String {
        String::from("Gotcha")
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::MCEvent(mcmsg) => {
                self.mark_components.update(&mcmsg, clipboard, &mut self.state);
                Command::none()
            }
            Message::SideBarMessage(ms) => {
                match ms {
                    SideBarMessage::CategoryMessage(ref cat_ms) => match cat_ms {
                        CategoryMessage::CategoryClicked(_) => {
                            self.mark_components.update(&MCMessage::CategoryClicked, clipboard, &mut self.state);
                        },
                        _ => ()
                    },
                };
                self.side_bar.update(ms, clipboard, &mut self.state).map(|event| Message::Events(event))
            },
            Message::HeaderMessage(m) => {
                self.header.update(m, clipboard, &mut self.state)
            }
            Message::Events(_) => {
                Command::none()
            },
            Message::Syncing(_) => {
                // println!()
                self.header.update(HeaderMessage::Loading, clipboard, &mut self.state)
            },
            Message::Synced(_) => {
                self.header.update(HeaderMessage::Loaded, clipboard, &mut self.state);
                self.state = State::LoadItems(0, Some(BookmarkCategory::default()), None);
                Command::none()
            }
        }
    }
    fn view(&mut self) -> Element<Self::Message> {
        let mut content = Column::new()
            .width(Length::Fill)
            .spacing(10);
        content = content.push(self.header.view().map(|ms| Message::HeaderMessage(ms)));
        let sidebar = Scrollable::new(&mut self.sidebar_scroll).width(Length::FillPortion(3))
        .push(
            self.side_bar.view().map(|message| Message::SideBarMessage(message))
        )
        .push(Space::new(Length::Fill, Length::Units(1500)))
        ;
        let body = Scrollable::new(&mut self.body_scroll)
            .width(Length::FillPortion(6))
            .push(
                self.mark_components
                    .view()
                    .map(|message| Message::MCEvent(message)),
            );
        content = content.push(
            Row::new()
                .push(
                    sidebar,
                )
                .push(body)
                .spacing(20),
        );
        Container::new(content).style(style::Surface).width(Length::Fill).into()
    }
    fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            State::LoadItems(i, cat, search) => {
                subscriptions::marks::load_marks(i, cat.to_owned(), search.to_owned()).map(|mark| Message::MCEvent(MCMessage::LoadMark(mark)))
            },
            _ => Subscription::none()
        }
    }

    fn mode(&self) -> iced::window::Mode {
        iced::window::Mode::Windowed
    }

    fn background_color(&self) -> iced::Color {
        iced::Color::WHITE
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }

}



