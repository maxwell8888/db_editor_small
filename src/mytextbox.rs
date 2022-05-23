use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::pure::{
    button, column, container, pick_list, row, scrollable, slider, text, text_input,
    widget::{
        button,
        canvas::event::{self, Event},
        canvas::{
            self, Cache, Canvas, Cursor, Fill, Frame, Geometry, LineCap, Path, Program, Stroke,
        },
        container, text_input, Button, Column, Row, Text, TextInput,
    },
    Element, Sandbox,
};
use iced::tooltip::{self, Tooltip};
// use iced::widget::{button, container, pick_list, scrollable, slider, text_input, Scrollable};
// use iced::Application;
use iced::pure::Application;
use iced::Font;
use iced::{
    clipboard, window, Background, Checkbox, Color, Command, Container, Length, PickList, Point,
    Radio, Rectangle, Settings, Size, Slider, Space, Vector,
};
// VerticalAlignment, Align, HorizontalAlignment,
use iced::{executor, mouse};

#[derive(Clone)]
pub enum MyTextboxMessage {
    InputChanged(String),
}

// #[derive(Default)]
pub struct MyTextbox {
    pub value: String,
    width: Length,
}
impl MyTextbox {
    pub fn new(value: String) -> Self {
        Self {
            value,
            width: Length::Units(200),
        }
    }
    pub fn update(&mut self, message: MyTextboxMessage) {
        match message {
            MyTextboxMessage::InputChanged(val) => self.value = val,
        }
    }
    pub fn view(&self) -> Element<MyTextboxMessage> {
        container(
            text_input("empty", &self.value, MyTextboxMessage::InputChanged)
                .padding(5)
                .style(TextboxStyle {}),
        )
        .width(self.width)
        .height(Length::Units(30))
        .into()
    }

    pub fn width(mut self, length: Length) -> Self {
        self.width = length;
        self
    }
}

struct TextboxStyle {}

impl text_input::StyleSheet for TextboxStyle {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: iced::Background::Color(iced::Color::TRANSPARENT),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.7, 0.7, 0.7),
            ..Default::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: iced::Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn selection_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.8, 0.8, 1.0)
    }

    // other methods in Stylesheet have a default impl
}
