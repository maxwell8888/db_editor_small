
use crate::colors;
use iced::pure::widget::container;
use iced::Color;

#[derive(Copy, Default, Clone)]
pub struct ConfigData {
    pub col_width: u16,
    pub row_height: u16,
}

pub struct NiceBox;
impl container::StyleSheet for NiceBox {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            background: Some(colors::LIGHT_BLUE.into()),
            border_radius: 12.0,
            border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}
pub struct BlackBorder;
impl container::StyleSheet for BlackBorder {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            // background: Some(Color::from_rgb(0.11, 0.42, 0.87).into()),
            // border_radius: 12.0,
            border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}

pub struct BlueBackground;
impl container::StyleSheet for BlueBackground {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            background: Some(Color::from_rgb(138.0 / 255.0, 187.0 / 255.0, 255.0 / 255.0).into()),
            // border_radius: 12.0,
            // border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            // border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}

pub struct LightBlueBackground;

impl container::StyleSheet for LightBlueBackground {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            background: Some(Color::from_rgb(0.8, 0.9, 1.0).into()),
            // border_radius: 12.0,
            // border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            // border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}

pub struct GreenBackground;

impl container::StyleSheet for GreenBackground {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            background: Some(Color::from_rgb(138.0 / 255.0, 187.0 / 255.0, 140.0 / 255.0).into()),
            // border_radius: 12.0,
            // border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            // border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}
pub struct LightGreenBackground;

impl container::StyleSheet for LightGreenBackground {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            background: Some(Color::from_rgb(207.0 / 255.0, 255.0 / 255.0, 207.0 / 255.0).into()),
            // border_radius: 12.0,
            // border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            // border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}

pub struct GreyBackground;

impl container::StyleSheet for GreyBackground {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            background: Some(Color::from_rgb(0.5, 0.5, 0.5).into()),
            // border_radius: 12.0,
            // border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            // border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}

pub struct LightGreyBackground;

impl container::StyleSheet for LightGreyBackground {
    fn style(&self) -> container::Style {
        container::Style {
            // text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
            background: Some(Color::from_rgb(0.8, 0.8, 0.8).into()),
            // border_radius: 12.0,
            // border_width: 2.0,
            // border_color: Color::from_rgb(0.11, 0.42, 0.87),
            // border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}
