use std::fmt::Debug;

use iced::{Align, Element, Length};

use secalc_core::grid::GridCalculator;

use crate::data_bind::{DataBind, DataBindMessage};
use crate::view::{col, lbl, row};

macro_rules! create_option_input {
  ($label_width:expr; $input_width:expr; $($field:ident, $type:ty, $message:ident, $label:expr, $format:expr, $unit:expr);*) => {
    pub struct OptionInput {
      $($field: DataBind<$type>,)*
    }

    impl OptionInput {
      pub fn new(default_calculator: &GridCalculator, loaded_calculator: &GridCalculator) -> Self {
        Self {
          $($field: DataBind::new(default_calculator.$field, format!($format, default_calculator.$field), $input_width, $unit, format!("{}", loaded_calculator.$field)),)*
        }
      }
    }

    #[derive(Clone, Debug)]
    pub enum OptionInputMessage {
      $($message(DataBindMessage),)*
    }

    impl OptionInput {
      pub fn update(&mut self, message: OptionInputMessage, calc: &mut GridCalculator) {
        match message {
          $(OptionInputMessage::$message(m) => self.$field.update(m, &mut calc.$field),)*
        }
      }

      pub fn reload(&mut self, calc: &GridCalculator) {
        $(self.$field.reload(format!("{}", calc.$field));)*
      }

      pub fn view(&mut self) -> Element<OptionInputMessage> {
        col()
          $(.push(row().push(lbl($label).width($label_width)).align_items(Align::Center).push(self.$field.view().map(move |s| OptionInputMessage::$message(s)))))*
          .into()
      }
    }
  }
}

create_option_input!(Length::Units(200); Length::Units(95);
  gravity_multiplier, f64, GravityMultiplier, "Gravity Multiplier", "{:.1}", "*";
  container_multiplier, f64, ContainerMultiplier, "Container Multiplier", "{:.1}", "*";
  planetary_influence, f64, PlanetaryInfluence, "Planetary Influence", "{:.1}", "*";
  additional_mass, f64, AdditionalMass, "Additional Mass", "{}", "kg";
  ice_only_fill, f64, IceOnlyFill, "Ice-only-fill", "{:.1}", "%";
  ore_only_fill, f64, OreOnlyFill, "Ore-only-fill", "{:.1}", "%";
  any_fill_with_ice, f64, AnyFillWithIce, "Any-fill with Ice", "{:.1}", "%";
  any_fill_with_ore, f64, AnyFillWithOre, "Any-fill with Ore", "{:.1}", "%";
  any_fill_with_steel_plates, f64, AnyFillWithSteelPlates, "Any-fill with Steel Plates", "{:.1}", "%"
);
