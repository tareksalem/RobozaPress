use iced::{container, Background, Color, button, Vector, text_input};

pub const PRIMARY_COLOR: Color = Color::from_rgba(34 as f32 / 255.0, 40 as f32 / 255.0, 49 as f32 / 255.0, 1.0);
pub const LIGHT_PRIMARY_COLOR: Color = Color::from_rgba(57 as f32 / 255.0, 62 as f32 / 255.0, 70 as f32 / 255.0, 0.8);
pub const LIGHT_COLOR: Color = Color::from_rgba(238 as f32 / 255.0, 238 as f32 / 255.0, 238 as f32 / 255.0, 1.0);
pub const TEXT_COLOR: Color =  LIGHT_COLOR;
const BORDER_COLOR: Color =
    Color::from_rgba(0 as f32 / 255.0, 0 as f32 / 255.0, 17 as f32 / 255.0, 0.29);
pub const PINK: Color = Color::from_rgba(245 as f32 / 255.0, 40 as f32 / 255.0,145 as f32 / 255.0,0.8);
pub const SECONDARY_COLOR: Color =  Color::from_rgba(255 as f32 / 255.0, 255 as f32 / 255.0, 145 as f32 / 255.0, 0.8);
pub const GREEN: Color = Color::from_rgba(0 as f32 / 255.0,174 as f32 / 255.0,145 as f32 / 255.0,0.8);
pub const LIGHT_GREEN: Color = Color::from_rgba(0 as f32 / 255.0,199 as f32 / 255.0,0 as f32 / 255.0,0.34);
pub const LIGHT_SECONDARY_COLOR: Color = Color::from_rgba(177 as f32 / 255.0,199 as f32 / 255.0,0 as f32 / 255.0,0.34);
pub const LIGHT_ACCUA: Color = Color::from_rgba(60 as f32 / 255.0, 252 as f32 / 255.0, 255 as f32 / 255.0, 0.66);
pub const ACCENT: Color = Color::from_rgba(
    31 as f32 / 255.0,
    117 as f32 / 255.0,
    255 as f32 / 255.0,
    0.6
);

pub const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);

pub const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);
pub struct FeedCard;

impl container::StyleSheet for FeedCard {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(LIGHT_PRIMARY_COLOR)),
            text_color: Some(TEXT_COLOR),
            border_color: BORDER_COLOR,
            border_width: 1.0,
            border_radius: 10.0,
            // ..container::Style::default()
        }
    }
}

pub struct Surface;

impl container::StyleSheet for Surface {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(PRIMARY_COLOR)),
            ..container::Style::default()
        }
    }
}
#[derive(Debug, Clone)]
pub enum ButtonStylingType{
    Random,
    Index(usize),
    Specific(ButtonStyle)
}
#[derive(Debug, Clone)]
pub struct ButtonStyle{
    background_color: Color,
    text_color: Color,
    border_radius: f32
}
pub struct StyledButton {
    pub style: ButtonStyle
}
impl StyledButton{
    pub fn styles() -> [ButtonStyle; 1] {
        let default_border_radius = 15.0;
        let button_styles: [ButtonStyle; 1] = [
            // ButtonStyle{
            //     background_color: PINK,
            //     text_color: TEXT_COLOR,
            //     border_radius: default_border_radius
            // },
            ButtonStyle{
                background_color: SECONDARY_COLOR,
                text_color: LIGHT_PRIMARY_COLOR,
                border_radius: default_border_radius
            },
            // ButtonStyle{
            //     background_color: LIGHT_ACCUA,
            //     text_color: DARK,
            //     border_radius: default_border_radius
            // },
            // ButtonStyle{
            //     background_color: GREEN,
            //     text_color: TEXT_COLOR,
            //     border_radius: default_border_radius
            // },
            // ButtonStyle{
            //     background_color: LIGHT_GREEN,
            //     text_color: TEXT_COLOR,
            //     border_radius: default_border_radius
            // },
            // ButtonStyle{
            //     background_color: WHITE,
            //     text_color: DARK,
            //     border_radius: default_border_radius
            // }
        ];
        button_styles
    }
    pub fn new(styling_type: ButtonStylingType) -> StyledButton {
        let styles = Self::styles();
        match styling_type {
            ButtonStylingType::Random => {
                let index = (rand::random::<f32>() * styles.len() as f32).floor() as usize;
                StyledButton{
                    style: styles[index].clone()
                }
            },
            ButtonStylingType::Index(index) => {
                let mut index = index;
                if index > styles.len() -1 {
                    index = index % styles.len();
                }
                StyledButton{
                    style: styles[index as usize].clone()
                }
            },
            ButtonStylingType::Specific(style) => {
                StyledButton{
                    style
                }
            },
        }
    }
    pub fn accent() -> Self {
        Self::new(ButtonStylingType::Specific(
            ButtonStyle{
                background_color: ACCENT,
                text_color: TEXT_COLOR,
                border_radius: 0.0
            }
        ))
    }
    pub fn primary() -> Self{
        Self::new(ButtonStylingType::Specific(
            ButtonStyle{
                background_color: LIGHT_PRIMARY_COLOR,
                text_color: TEXT_COLOR,
                border_radius: 1.0
            }
        ))
    }
}
impl button::StyleSheet for StyledButton{
    fn active(&self) -> button::Style {
        button::Style{
            text_color: self.style.text_color,
            background: Some(Background::Color(self.style.background_color)),
            shadow_offset: Vector::new(0.1, 1.0),
            border_width: 1.0,
            border_color: BORDER_COLOR,
            border_radius: self.style.border_radius,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();
        let style = &self.style;
        button::Style {
            background: Some(Background::Color(Color::from_rgba(style.background_color.r, style.background_color.g, style.background_color.b, 1.0))),
            shadow_offset: active.shadow_offset + iced::Vector::new(0.0, 4.0),
            ..active
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::default(),
            background: Some(Background::Color(self.style.background_color)),
            ..self.active()
        }
    }

    fn disabled(&self) -> button::Style {
        let active = self.active();

        button::Style {
            shadow_offset: iced::Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

pub struct Header;

impl container::StyleSheet for Header{
    fn style(&self) -> container::Style {
        container::Style{
            background: Some(Background::Color(LIGHT_PRIMARY_COLOR)),
            border_radius: 1.0,
            border_width: 1.0,
            border_color: BORDER_COLOR,
            ..container::Style::default()
        }
    }
}
struct TextInputStyle{
    background_color: Color,
    text_color: Color,
    border_color: Color,
    selection_color: Color,
    border_width: f32,
    border_radius: f32,
}
pub struct StyledTextInput{
    style: TextInputStyle
}
impl StyledTextInput{
    pub fn new() -> Self{
        Self{
            style: TextInputStyle{
                background_color: LIGHT_PRIMARY_COLOR,
                text_color: TEXT_COLOR,
                border_color: Color::TRANSPARENT,
                selection_color: Color::WHITE,
                border_width: 0.0,
                border_radius: 0.0
            }
        }
    }
}
impl text_input::StyleSheet for StyledTextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style{
             background: Background::Color(self.style.background_color),
             border_color: self.style.border_color,
             border_width: self.style.border_width,
             border_radius: self.style.border_radius,
        }
    }

    fn focused(&self) -> text_input::Style {
        let style = &self.style;
        let background_color = style.background_color;
        text_input::Style{
            background: Background::Color(Color::from_rgba(background_color.r, background_color.g, background_color.b, 1.0)),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        self.style.text_color
    }

    fn value_color(&self) -> Color {
        self.style.text_color
    }

    fn selection_color(&self) -> Color {
        self.style.selection_color
    }

    fn hovered(&self) -> text_input::Style {
        self.focused()
    }
}