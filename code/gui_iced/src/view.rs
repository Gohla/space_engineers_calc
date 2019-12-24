use iced::{Button, button, Color, Column, HorizontalAlignment, Length, Row, Scrollable, scrollable, Text, text_input, TextInput, VerticalAlignment};

#[inline]
pub fn row<'a, M>() -> Row<'a, M> { Row::new().width(Length::Shrink) }

#[inline]
pub fn col<'a, M>() -> Column<'a, M> { Column::new().width(Length::Shrink) }

#[inline]
pub fn scl<M>(state: &mut scrollable::State) -> Scrollable<M> { Scrollable::new(state).width(Length::Shrink) }

pub const TXT_SIZE: u16 = 18;
pub const H3_SIZE: u16 = 20;
pub const H2_SIZE: u16 = 24;
pub const H1_SIZE: u16 = 32;

#[inline]
pub fn lbl<L: Into<String>>(label: L) -> Text { Text::new(label).size(TXT_SIZE).width(Length::Shrink) }

#[inline]
pub fn val<L: Into<String>>(label: L) -> Text { Text::new(label).size(TXT_SIZE).width(Length::Shrink).horizontal_alignment(HorizontalAlignment::Right) }

#[inline]
pub fn h3<L: Into<String>>(label: L) -> Text { Text::new(label).size(H3_SIZE).width(Length::Shrink) }

#[inline]
pub fn h2<L: Into<String>>(label: L) -> Text { Text::new(label).size(H2_SIZE).width(Length::Shrink) }

#[inline]
pub fn h1<L: Into<String>>(label: L) -> Text { Text::new(label).size(H1_SIZE).width(Length::Shrink) }

#[inline]
pub fn text_input<'a, M, F: 'static + Fn(String) -> M>(width: Length, state: &'a mut text_input::State, placeholder: &str, value: &str, on_change: F) -> TextInput<'a, M> {
  TextInput::new(state, placeholder, value, on_change)
    .width(width)
    .padding(2)
    .size(TXT_SIZE)
}

#[inline]
pub fn button<M, L: Into<String>>(state: &mut button::State, label: L) -> Button<M> {
  Button::new(state, h2(label).vertical_alignment(VerticalAlignment::Center))
    .padding(2)
    .min_width(50)
    .background(Color::from_rgb(0.75, 0.75, 0.75))
    .border_radius(2)
}

#[inline]
pub fn empty() -> Text { Text::new(" ").size(TXT_SIZE).width(Length::Shrink) }


#[inline]
pub fn background_color() -> Color { Color::WHITE }

#[inline]
pub fn foreground_color() -> Color { Color::BLACK }

#[inline]
pub fn danger_color() -> Color { Color::from_rgb(0.8, 0.2, 0.2) }
