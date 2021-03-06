use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use gdk::ModifierType;
use gtk::{Align, Application, ApplicationWindow, Button, Entry, Grid, InputPurpose, Label};
use gtk::prelude::*;
use snafu::{ResultExt, Snafu};

use secalc_core::grid::{GridCalculator, Direction};
use secalc_core::data::blocks::{Block, BlockId, Blocks};
use secalc_core::data::Data;

use crate::gui::dialog::{ErrorDialogResultExt, FileDialog};

#[derive(Debug, Snafu)]
pub enum OpenError {
  #[snafu(display("Could not open file '{}' for reading: {}", file_path.display(), source))]
  OpenFile { file_path: PathBuf, source: std::io::Error, },
  #[snafu(display("Could not deserialize data from file '{}': {}", file_path.display(), source))]
  OpenDeserialize { file_path: PathBuf, source: secalc_core::grid::ReadError, },
}

#[derive(Debug, Snafu)]
pub enum SaveError {
  #[snafu(display("Could not open file '{}' for writing: {}", file_path.display(), source))]
  SaveFile { file_path: PathBuf, source: std::io::Error, },
  #[snafu(display("Could not serialize data to file '{}': {}", file_path.display(), source))]
  SaveSerialize { file_path: PathBuf, source: secalc_core::grid::WriteError, },
}

pub struct MainWindow {
  window: ApplicationWindow,

  open: Button,
  save: Button,
  save_as: Button,

  gravity_multiplier: Entry,
  container_multiplier: Entry,
  planetary_influence: Entry,
  additional_mass: Entry,
  ice_only_fill: Entry,
  ore_only_fill: Entry,
  any_fill_with_ice: Entry,
  any_fill_with_ore: Entry,
  any_fill_with_steel_plates: Entry,

  volume_mass_input_small: Grid,
  volume_mass_input_large: Grid,
  total_volume_any: Label,
  total_volume_ore: Label,
  total_volume_ice: Label,
  total_volume_ore_only: Label,
  total_volume_ice_only: Label,
  total_mass_empty: Label,
  total_mass_filled: Label,
  total_items_ice: Label,
  total_items_ore: Label,
  total_items_steel_plates: Label,

  acceleration_input_small: Grid,
  acceleration_input_large: Grid,
  thrusters: HashMap<Direction, ThrusterWidgets>,

  power_input_small: Grid,
  power_input_large: Grid,
  power_generation: Label,
  power_capacity_battery: Label,
  power_consumption_idle: Label,
  power_consumption_misc: Label,
  power_consumption_upto_jump_drive: Label,
  power_consumption_upto_generator: Label,
  power_consumption_upto_up_down_thruster: Label,
  power_consumption_upto_front_back_thruster: Label,
  power_consumption_upto_left_right_thruster: Label,
  power_consumption_upto_battery: Label,
  power_balance_idle: Label,
  power_balance_misc: Label,
  power_balance_upto_jump_drive: Label,
  power_balance_upto_generator: Label,
  power_balance_upto_up_down_thruster: Label,
  power_balance_upto_front_back_thruster: Label,
  power_balance_upto_left_right_thruster: Label,
  power_balance_upto_battery: Label,
  power_duration_idle: Label,
  power_duration_misc: Label,
  power_duration_upto_jump_drive: Label,
  power_duration_upto_generator: Label,
  power_duration_upto_up_down_thruster: Label,
  power_duration_upto_front_back_thruster: Label,
  power_duration_upto_left_right_thruster: Label,
  power_duration_upto_battery: Label,

  hydrogen_input_small: Grid,
  hydrogen_input_large: Grid,
  hydrogen_generation: Label,
  hydrogen_capacity_engine: Label,
  hydrogen_capacity_tank: Label,
  hydrogen_consumption_idle: Label,
  hydrogen_consumption_engine: Label,
  hydrogen_consumption_upto_up_down_thruster: Label,
  hydrogen_consumption_upto_front_back_thruster: Label,
  hydrogen_consumption_upto_left_right_thruster: Label,
  hydrogen_balance_idle: Label,
  hydrogen_balance_engine: Label,
  hydrogen_balance_upto_up_down_thruster: Label,
  hydrogen_balance_upto_front_back_thruster: Label,
  hydrogen_balance_upto_left_right_thruster: Label,
  hydrogen_duration_idle: Label,
  hydrogen_duration_engine: Label,
  hydrogen_duration_upto_up_down_thruster: Label,
  hydrogen_duration_upto_front_back_thruster: Label,
  hydrogen_duration_upto_left_right_thruster: Label,

  data: Data,
  state: RefCell<State> /* RefCell to support mutability for Rc<Self> in closures. */,
  block_entries: RefCell<BlockEntries> /* RefCell to support mutability for Rc<Self>. */,
}

struct ThrusterWidgets {
  force: Label,
  acceleration_empty_no_gravity: Label,
  acceleration_filled_no_gravity: Label,
  acceleration_empty_gravity: Label,
  acceleration_filled_gravity: Label,
}

struct State {
  current_dir_path: Option<PathBuf>,
  current_file_path: Option<PathBuf>,
  calculator: GridCalculator,
}

struct BlockEntries {
  entries: HashMap<BlockId, Entry>,
  up_entries: HashMap<BlockId, Entry>,
  down_entries: HashMap<BlockId, Entry>,
  front_entries: HashMap<BlockId, Entry>,
  back_entries: HashMap<BlockId, Entry>,
  left_entries: HashMap<BlockId, Entry>,
  right_entries: HashMap<BlockId, Entry>,
}

impl BlockEntries {
  fn iter_entries(&self) -> impl Iterator<Item=&Entry> {
    vec![
      self.entries.values(),
      self.up_entries.values(),
      self.down_entries.values(),
      self.front_entries.values(),
      self.back_entries.values(),
      self.left_entries.values(),
      self.right_entries.values(),
    ].into_iter().flat_map(|it| it)
  }
}

impl MainWindow {
  pub fn new(data: Data) -> Rc<Self> {
    let glade_src = include_str!("main_window.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let window = builder.get_object("application_window").unwrap();

    let open = builder.get_object("open").unwrap();
    let save = builder.get_object("save").unwrap();
    let save_as = builder.get_object("save_as").unwrap();

    let gravity_multiplier = builder.get_object("gravity_multiplier").unwrap();
    let container_multiplier = builder.get_object("container_multiplier").unwrap();
    let planetary_influence = builder.get_object("planetary_influence").unwrap();
    let additional_mass = builder.get_object("additional_mass").unwrap();
    let ice_only_fill = builder.get_object("ice_only_fill").unwrap();
    let ore_only_fill = builder.get_object("ore_only_fill").unwrap();
    let any_fill_with_ice = builder.get_object("any_fill_with_ice").unwrap();
    let any_fill_with_ore = builder.get_object("any_fill_with_ore").unwrap();
    let any_fill_with_steel_plates = builder.get_object("any_fill_with_steel_plates").unwrap();

    let volume_mass_input_small = builder.get_object("volume_mass_input_small").unwrap();
    Self::cleanup_glade_grid(&volume_mass_input_small);
    let volume_mass_input_large = builder.get_object("volume_mass_input_large").unwrap();
    Self::cleanup_glade_grid(&volume_mass_input_large);
    let total_volume_any = builder.get_object("total_volume_any").unwrap();
    let total_volume_ore = builder.get_object("total_volume_ore").unwrap();
    let total_volume_ice = builder.get_object("total_volume_ice").unwrap();
    let total_volume_ore_only = builder.get_object("total_volume_ore_only").unwrap();
    let total_volume_ice_only = builder.get_object("total_volume_ice_only").unwrap();
    let total_mass_empty = builder.get_object("total_mass_empty").unwrap();
    let total_mass_filled = builder.get_object("total_mass_filled").unwrap();
    let total_items_ice = builder.get_object("total_items_ice").unwrap();
    let total_items_ore = builder.get_object("total_items_ore").unwrap();
    let total_items_steel_plates = builder.get_object("total_items_steel_plates").unwrap();

    let acceleration_input_small = builder.get_object("acceleration_input_small").unwrap();
    let acceleration_input_large = builder.get_object("acceleration_input_large").unwrap();
    let mut thrusters = HashMap::default();
    for side in Direction::iter() {
      let side = *side;
      let id_prefix = match side {
        Direction::Up => "up",
        Direction::Down => "down",
        Direction::Front => "front",
        Direction::Back => "back",
        Direction::Left => "left",
        Direction::Right => "right",
      };
      let force = builder.get_object(&(id_prefix.to_string() + "_force")).unwrap();
      let acceleration_empty_no_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_empty_no_gravity")).unwrap();
      let acceleration_filled_no_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_filled_no_gravity")).unwrap();
      let acceleration_empty_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_empty_gravity")).unwrap();
      let acceleration_filled_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_filled_gravity")).unwrap();
      let thruster_widgets = ThrusterWidgets {
        force,
        acceleration_empty_no_gravity,
        acceleration_filled_no_gravity,
        acceleration_empty_gravity,
        acceleration_filled_gravity,
      };
      thrusters.insert(side, thruster_widgets);
    }

    let power_input_small = builder.get_object("power_input_small").unwrap();
    Self::cleanup_glade_grid(&power_input_small);
    let power_input_large = builder.get_object("power_input_large").unwrap();
    Self::cleanup_glade_grid(&power_input_large);
    let power_generation = builder.get_object("power_generation").unwrap();
    let power_capacity_battery = builder.get_object("power_capacity_battery").unwrap();
    let power_consumption_idle = builder.get_object("power_consumption_idle").unwrap();
    let power_consumption_misc = builder.get_object("power_consumption_misc").unwrap();
    let power_consumption_upto_jump_drive = builder.get_object("power_consumption_upto_jump_drive").unwrap();
    let power_consumption_upto_generator = builder.get_object("power_consumption_upto_generator").unwrap();
    let power_consumption_upto_up_down_thruster = builder.get_object("power_consumption_upto_up_down_thruster").unwrap();
    let power_consumption_upto_front_back_thruster = builder.get_object("power_consumption_upto_front_back_thruster").unwrap();
    let power_consumption_upto_left_right_thruster = builder.get_object("power_consumption_upto_left_right_thruster").unwrap();
    let power_consumption_upto_battery = builder.get_object("power_consumption_upto_battery").unwrap();
    let power_balance_idle = builder.get_object("power_balance_idle").unwrap();
    let power_balance_misc = builder.get_object("power_balance_misc").unwrap();
    let power_balance_upto_jump_drive = builder.get_object("power_balance_upto_jump_drive").unwrap();
    let power_balance_upto_generator = builder.get_object("power_balance_upto_generator").unwrap();
    let power_balance_upto_up_down_thruster = builder.get_object("power_balance_upto_up_down_thruster").unwrap();
    let power_balance_upto_front_back_thruster = builder.get_object("power_balance_upto_front_back_thruster").unwrap();
    let power_balance_upto_left_right_thruster = builder.get_object("power_balance_upto_left_right_thruster").unwrap();
    let power_balance_upto_battery = builder.get_object("power_balance_upto_battery").unwrap();
    let power_duration_idle = builder.get_object("power_duration_idle").unwrap();
    let power_duration_misc = builder.get_object("power_duration_misc").unwrap();
    let power_duration_upto_jump_drive = builder.get_object("power_duration_upto_jump_drive").unwrap();
    let power_duration_upto_generator = builder.get_object("power_duration_upto_generator").unwrap();
    let power_duration_upto_up_down_thruster = builder.get_object("power_duration_upto_up_down_thruster").unwrap();
    let power_duration_upto_front_back_thruster = builder.get_object("power_duration_upto_front_back_thruster").unwrap();
    let power_duration_upto_left_right_thruster = builder.get_object("power_duration_upto_left_right_thruster").unwrap();
    let power_duration_upto_battery = builder.get_object("power_duration_upto_battery").unwrap();

    let hydrogen_input_small = builder.get_object("hydrogen_input_small").unwrap();
    Self::cleanup_glade_grid(&hydrogen_input_small);
    let hydrogen_input_large = builder.get_object("hydrogen_input_large").unwrap();
    Self::cleanup_glade_grid(&hydrogen_input_large);
    let hydrogen_generation = builder.get_object("hydrogen_generation").unwrap();
    let hydrogen_capacity_engine = builder.get_object("hydrogen_capacity_engine").unwrap();
    let hydrogen_capacity_tank = builder.get_object("hydrogen_capacity_tank").unwrap();
    let hydrogen_consumption_idle = builder.get_object("hydrogen_consumption_idle").unwrap();
    let hydrogen_consumption_engine = builder.get_object("hydrogen_consumption_engine").unwrap();
    let hydrogen_consumption_upto_up_down_thruster = builder.get_object("hydrogen_consumption_upto_up_down_thruster").unwrap();
    let hydrogen_consumption_upto_front_back_thruster = builder.get_object("hydrogen_consumption_upto_front_back_thruster").unwrap();
    let hydrogen_consumption_upto_left_right_thruster = builder.get_object("hydrogen_consumption_upto_left_right_thruster").unwrap();
    let hydrogen_balance_idle = builder.get_object("hydrogen_balance_idle").unwrap();
    let hydrogen_balance_engine = builder.get_object("hydrogen_balance_engine").unwrap();
    let hydrogen_balance_upto_up_down_thruster = builder.get_object("hydrogen_balance_upto_up_down_thruster").unwrap();
    let hydrogen_balance_upto_front_back_thruster = builder.get_object("hydrogen_balance_upto_front_back_thruster").unwrap();
    let hydrogen_balance_upto_left_right_thruster = builder.get_object("hydrogen_balance_upto_left_right_thruster").unwrap();
    let hydrogen_duration_idle = builder.get_object("hydrogen_duration_idle").unwrap();
    let hydrogen_duration_engine = builder.get_object("hydrogen_duration_engine").unwrap();
    let hydrogen_duration_upto_up_down_thruster = builder.get_object("hydrogen_duration_upto_up_down_thruster").unwrap();
    let hydrogen_duration_upto_front_back_thruster = builder.get_object("hydrogen_duration_upto_front_back_thruster").unwrap();
    let hydrogen_duration_upto_left_right_thruster = builder.get_object("hydrogen_duration_upto_left_right_thruster").unwrap();

    let state = RefCell::new(State {
      current_dir_path: env::current_dir().ok(),
      current_file_path: None,
      calculator: GridCalculator::default()
    });
    let block_entries = RefCell::new(BlockEntries {
      entries: Default::default(),
      up_entries: Default::default(),
      down_entries: Default::default(),
      front_entries: Default::default(),
      back_entries: Default::default(),
      left_entries: Default::default(),
      right_entries: Default::default()
    });

    let main_window = Rc::new(MainWindow {
      window,

      open,
      save,
      save_as,

      gravity_multiplier,
      container_multiplier,
      planetary_influence,
      additional_mass,
      ice_only_fill,
      ore_only_fill,
      any_fill_with_ice,
      any_fill_with_ore,
      any_fill_with_steel_plates,

      volume_mass_input_small,
      volume_mass_input_large,
      total_volume_any,
      total_volume_ore,
      total_volume_ice,
      total_volume_ore_only,
      total_volume_ice_only,
      total_mass_empty,
      total_mass_filled,
      total_items_ice,
      total_items_ore,
      total_items_steel_plates,

      acceleration_input_small,
      acceleration_input_large,
      thrusters,

      power_input_small,
      power_input_large,
      power_generation,
      power_capacity_battery,
      power_consumption_idle,
      power_consumption_misc,
      power_consumption_upto_jump_drive,
      power_consumption_upto_generator,
      power_consumption_upto_up_down_thruster,
      power_consumption_upto_front_back_thruster,
      power_consumption_upto_left_right_thruster,
      power_consumption_upto_battery,
      power_balance_idle,
      power_balance_misc,
      power_balance_upto_jump_drive,
      power_balance_upto_generator,
      power_balance_upto_up_down_thruster,
      power_balance_upto_front_back_thruster,
      power_balance_upto_left_right_thruster,
      power_balance_upto_battery,
      power_duration_idle,
      power_duration_misc,
      power_duration_upto_jump_drive,
      power_duration_upto_generator,
      power_duration_upto_up_down_thruster,
      power_duration_upto_front_back_thruster,
      power_duration_upto_left_right_thruster,
      power_duration_upto_battery,

      hydrogen_input_small,
      hydrogen_input_large,
      hydrogen_generation,
      hydrogen_capacity_engine,
      hydrogen_capacity_tank,
      hydrogen_consumption_idle,
      hydrogen_consumption_engine,
      hydrogen_consumption_upto_up_down_thruster,
      hydrogen_consumption_upto_front_back_thruster,
      hydrogen_consumption_upto_left_right_thruster,
      hydrogen_balance_idle,
      hydrogen_balance_engine,
      hydrogen_balance_upto_up_down_thruster,
      hydrogen_balance_upto_front_back_thruster,
      hydrogen_balance_upto_left_right_thruster,
      hydrogen_duration_idle,
      hydrogen_duration_engine,
      hydrogen_duration_upto_up_down_thruster,
      hydrogen_duration_upto_front_back_thruster,
      hydrogen_duration_upto_left_right_thruster,

      data,
      state,
      block_entries,
    });
    main_window.clone().initialize();
    main_window.clone().recalculate();
    main_window
  }

  fn initialize(self: Rc<Self>) {
    let self_cloned = self.clone();
    self.open.connect_clicked(move |_| {
      self_cloned.open();
    });

    let self_cloned = self.clone();
    self.save.connect_clicked(move |_| {
      self_cloned.save_or_save_as();
    });

    let self_cloned = self.clone();
    self.save_as.connect_clicked(move |_| {
      self_cloned.save_as();
    });

    let self_cloned = self.clone();
    self.window.connect_key_press_event(move |_, event_key| {
      let ctrl_down = event_key.get_state().contains(ModifierType::CONTROL_MASK);
      match event_key.get_keyval() {
        key if key == 's' as u32 && ctrl_down => {
          self_cloned.save_or_save_as();
        }
        key if key == 'o' as u32 && ctrl_down => {
          self_cloned.open();
        }
        _ => {}
      };
      Inhibit(false)
    });

    self.gravity_multiplier.set_and_recalc_on_change(&self, 1.0, |c| &mut c.gravity_multiplier);
    self.container_multiplier.set_and_recalc_on_change(&self, 1.0, |c| &mut c.container_multiplier);
    self.planetary_influence.set_and_recalc_on_change(&self, 1.0, |c| &mut c.planetary_influence);
    self.additional_mass.set_and_recalc_on_change(&self, 0.0, |c| &mut c.additional_mass);
    self.ice_only_fill.set_and_recalc_on_change(&self, 100.0, |c| &mut c.ice_only_fill);
    self.ore_only_fill.set_and_recalc_on_change(&self, 100.0, |c| &mut c.ore_only_fill);
    self.any_fill_with_ice.set_and_recalc_on_change(&self, 0.0, |c| &mut c.any_fill_with_ice);
    self.any_fill_with_ore.set_and_recalc_on_change(&self, 0.0, |c| &mut c.any_fill_with_ore);
    self.any_fill_with_steel_plates.set_and_recalc_on_change(&self, 0.0, |c| &mut c.any_fill_with_steel_plates);

    // Volume & Mass
    self.clone().create_block_inputs(self.data.blocks.containers.values().filter(|c| c.details.store_any), &self.volume_mass_input_small, &self.volume_mass_input_large, |c| &mut c.blocks);
    self.clone().create_block_inputs(self.data.blocks.cockpits.values().filter(|c| c.details.has_inventory), &self.volume_mass_input_small, &self.volume_mass_input_large, |c| &mut c.blocks);
    // Acceleration
    self.clone().create_acceleration_block_inputs(self.data.blocks.thrusters.values(), &self.acceleration_input_small, &self.acceleration_input_large);
    // Power
    self.clone().create_block_inputs(self.data.blocks.hydrogen_engines.values(), &self.power_input_small, &self.power_input_large, |c| &mut c.blocks);
    self.clone().create_block_inputs(self.data.blocks.reactors.values(), &self.power_input_small, &self.power_input_large, |c| &mut c.blocks);
    self.clone().create_block_inputs(self.data.blocks.batteries.values(), &self.power_input_small, &self.power_input_large, |c| &mut c.blocks);
    // Hydrogen
    self.clone().create_block_inputs(self.data.blocks.generators.values(), &self.hydrogen_input_small, &self.hydrogen_input_large, |c| &mut c.blocks);
    self.clone().create_block_inputs(self.data.blocks.hydrogen_tanks.values(), &self.hydrogen_input_small, &self.hydrogen_input_large, |c| &mut c.blocks);
  }


  fn cleanup_glade_grid(grid: &Grid) {
    // Remove a column and 3 rows, because Glade always creates 3x3 grids.
    grid.remove_column(2);
    grid.remove_row(2);
    grid.remove_row(1);
    grid.remove_row(0);
  }

  fn create_block_inputs<'a, T: 'a, I, F>(
    self: Rc<Self>,
    iter: I,
    small_grid: &Grid,
    large_grid: &Grid,
    calculator_func: F
  ) where
    F: (Fn(&mut GridCalculator) -> &mut HashMap<BlockId, u64>) + 'static + Copy,
    I: Iterator<Item=&'a Block<T>>
  {
    let (small, large) = Blocks::small_and_large_sorted(iter);
    self.clone().create_block_input_grid(small, small_grid, calculator_func);
    self.create_block_input_grid(large, large_grid, calculator_func);
  }

  fn create_block_input_grid<T, F>(
    self: Rc<Self>,
    blocks: Vec<&Block<T>>,
    grid: &Grid,
    calculator_func: F
  ) where
    F: (Fn(&mut GridCalculator) -> &mut HashMap<BlockId, u64>) + 'static + Copy
  {
    let index_offset = grid.get_children().len() as i32;
    for (index, block) in blocks.into_iter().enumerate() {
      let index = index as i32 + index_offset;
      grid.insert_row(index as i32);
      let label = Self::create_static_label(block.name(&self.data.localization));
      grid.attach(&label, 0, index, 1, 1);
      let entry = Self::create_entry();
      entry.insert_and_recalc_on_change(&self, block.id.clone(), calculator_func);
      grid.attach(&entry, 1, index, 1, 1);
      self.block_entries.borrow_mut().entries.insert(block.id.clone(), entry.clone());
    }
  }


  fn create_acceleration_block_inputs<'a, T: 'a, I>(
    self: Rc<Self>,
    iter: I,
    small_grid: &Grid,
    large_grid: &Grid,
  ) where
    I: Iterator<Item=&'a Block<T>>
  {
    let (small, large) = Blocks::small_and_large_sorted(iter);
    self.clone().create_acceleration_block_input_grid(small, small_grid);
    self.create_acceleration_block_input_grid(large, large_grid);
  }

  fn create_acceleration_block_input_grid<T>(
    self: Rc<Self>,
    blocks: Vec<&Block<T>>,
    grid: &Grid,
  ) {
    let mut block_entries = self.block_entries.borrow_mut();
    for (index, block) in blocks.into_iter().enumerate() {
      let index = index as i32 + 1;
      grid.insert_row(index as i32);
      let label = Self::create_static_label(block.name(&self.data.localization));
      grid.attach(&label, 0, index, 1, 1);

      let entry_up = Self::create_entry();
      entry_up.insert_and_recalc_on_change(&self, block.id.clone(), |c| c.directional_blocks.get_mut(&Direction::Up).unwrap());
      grid.attach(&entry_up, 1, index, 1, 1);
      block_entries.up_entries.insert(block.id.clone(), entry_up.clone());

      let entry_down = Self::create_entry();
      entry_down.insert_and_recalc_on_change(&self, block.id.clone(), |c| c.directional_blocks.get_mut(&Direction::Down).unwrap());
      grid.attach(&entry_down, 2, index, 1, 1);
      block_entries.down_entries.insert(block.id.clone(), entry_down.clone());

      let entry_front = Self::create_entry();
      entry_front.insert_and_recalc_on_change(&self, block.id.clone(), |c| c.directional_blocks.get_mut(&Direction::Front).unwrap());
      grid.attach(&entry_front, 3, index, 1, 1);
      block_entries.front_entries.insert(block.id.clone(), entry_front.clone());

      let entry_back = Self::create_entry();
      entry_back.insert_and_recalc_on_change(&self, block.id.clone(), |c| c.directional_blocks.get_mut(&Direction::Back).unwrap());
      grid.attach(&entry_back, 4, index, 1, 1);
      block_entries.back_entries.insert(block.id.clone(), entry_back.clone());

      let entry_left = Self::create_entry();
      entry_left.insert_and_recalc_on_change(&self, block.id.clone(), |c| c.directional_blocks.get_mut(&Direction::Left).unwrap());
      grid.attach(&entry_left, 5, index, 1, 1);
      block_entries.left_entries.insert(block.id.clone(), entry_left.clone());

      let entry_right = Self::create_entry();
      entry_right.insert_and_recalc_on_change(&self, block.id.clone(), |c| c.directional_blocks.get_mut(&Direction::Right).unwrap());
      grid.attach(&entry_right, 6, index, 1, 1);
      block_entries.right_entries.insert(block.id.clone(), entry_right.clone());
    }
  }


  fn create_static_label(label: &str) -> Label {
    let label = Label::new(Some(label));
    label.set_halign(Align::Start);
    label
  }

  fn create_entry() -> Entry {
    let entry = Entry::new();
    entry.set_input_purpose(InputPurpose::Number);
    entry.set_placeholder_text(Some("0"));
    entry.set_width_chars(3);
    entry
  }


  fn recalculate(&self) {
    let calculated = self.state.borrow().calculator.calculate(&self.data);

    // Volume & Mass
    self.total_volume_any.set(calculated.total_volume_any);
    self.total_volume_ore.set(calculated.total_volume_ore);
    self.total_volume_ice.set(calculated.total_volume_ice);
    self.total_volume_ore_only.set(calculated.total_volume_ore_only);
    self.total_volume_ice_only.set(calculated.total_volume_ice_only);
    self.total_mass_empty.set(calculated.total_mass_empty);
    self.total_mass_filled.set(calculated.total_mass_filled);
    self.total_items_ice.set(calculated.total_items_ice);
    self.total_items_ore.set(calculated.total_items_ore);
    self.total_items_steel_plates.set(calculated.total_items_steel_plate);
    // Force & Acceleration
    for (side, a) in calculated.acceleration.iter() {
      let widgets = self.thrusters.get(side).unwrap();
      widgets.force.set(a.force);
      widgets.acceleration_empty_no_gravity.set(a.acceleration_empty_no_gravity);
      widgets.acceleration_filled_no_gravity.set(a.acceleration_filled_no_gravity);
      widgets.acceleration_empty_gravity.set(a.acceleration_empty_gravity);
      widgets.acceleration_filled_gravity.set(a.acceleration_filled_gravity);
    }
    // Power
    self.power_generation.set(calculated.power_generation);
    self.power_capacity_battery.set(calculated.power_capacity_battery);
    self.power_consumption_idle.set(calculated.power_idle.consumption);
    self.power_consumption_misc.set(calculated.power_misc.consumption);
    self.power_consumption_upto_jump_drive.set(calculated.power_upto_jump_drive.consumption);
    self.power_consumption_upto_generator.set(calculated.power_upto_generator.consumption);
    self.power_consumption_upto_up_down_thruster.set(calculated.power_upto_up_down_thruster.consumption);
    self.power_consumption_upto_front_back_thruster.set(calculated.power_upto_front_back_thruster.consumption);
    self.power_consumption_upto_left_right_thruster.set(calculated.power_upto_left_right_thruster.consumption);
    self.power_consumption_upto_battery.set(calculated.power_upto_battery.consumption);
    self.power_balance_idle.set(calculated.power_idle.balance);
    self.power_balance_misc.set(calculated.power_misc.balance);
    self.power_balance_upto_jump_drive.set(calculated.power_upto_jump_drive.balance);
    self.power_balance_upto_generator.set(calculated.power_upto_generator.balance);
    self.power_balance_upto_up_down_thruster.set(calculated.power_upto_up_down_thruster.balance);
    self.power_balance_upto_front_back_thruster.set(calculated.power_upto_front_back_thruster.balance);
    self.power_balance_upto_left_right_thruster.set(calculated.power_upto_left_right_thruster.balance);
    self.power_balance_upto_battery.set(calculated.power_upto_battery.balance);
    self.power_duration_idle.set(calculated.power_idle.duration);
    self.power_duration_misc.set(calculated.power_misc.duration);
    self.power_duration_upto_jump_drive.set(calculated.power_upto_jump_drive.duration);
    self.power_duration_upto_generator.set(calculated.power_upto_generator.duration);
    self.power_duration_upto_up_down_thruster.set(calculated.power_upto_up_down_thruster.duration);
    self.power_duration_upto_front_back_thruster.set(calculated.power_upto_front_back_thruster.duration);
    self.power_duration_upto_left_right_thruster.set(calculated.power_upto_left_right_thruster.duration);
    self.power_duration_upto_battery.set(calculated.power_upto_battery.duration);
    // Hydrogen
    self.hydrogen_generation.set(calculated.hydrogen_generation);
    self.hydrogen_capacity_engine.set(calculated.hydrogen_capacity_engine);
    self.hydrogen_capacity_tank.set(calculated.hydrogen_capacity_tank);
    self.hydrogen_consumption_idle.set(calculated.hydrogen_idle.consumption);
    self.hydrogen_consumption_engine.set(calculated.hydrogen_engine.consumption);
    self.hydrogen_consumption_upto_up_down_thruster.set(calculated.hydrogen_upto_up_down_thruster.consumption);
    self.hydrogen_consumption_upto_front_back_thruster.set(calculated.hydrogen_upto_front_back_thruster.consumption);
    self.hydrogen_consumption_upto_left_right_thruster.set(calculated.hydrogen_upto_left_right_thruster.consumption);
    self.hydrogen_balance_idle.set(calculated.hydrogen_idle.balance);
    self.hydrogen_balance_engine.set(calculated.hydrogen_engine.balance);
    self.hydrogen_balance_upto_up_down_thruster.set(calculated.hydrogen_upto_up_down_thruster.balance);
    self.hydrogen_balance_upto_front_back_thruster.set(calculated.hydrogen_upto_front_back_thruster.balance);
    self.hydrogen_balance_upto_left_right_thruster.set(calculated.hydrogen_upto_left_right_thruster.balance);
    self.hydrogen_duration_idle.set(calculated.hydrogen_idle.duration);
    self.hydrogen_duration_engine.set(calculated.hydrogen_engine.duration);
    self.hydrogen_duration_upto_up_down_thruster.set(calculated.hydrogen_upto_up_down_thruster.duration);
    self.hydrogen_duration_upto_front_back_thruster.set(calculated.hydrogen_upto_front_back_thruster.duration);
    self.hydrogen_duration_upto_left_right_thruster.set(calculated.hydrogen_upto_left_right_thruster.duration);
  }


  fn open(&self) {
    let dialog = FileDialog::new_open(&self.window, self.state.borrow().current_dir_path.as_ref());
    if let Some(file_path) = dialog.run() {
      self.process_open(file_path).show_error_as_dialog(&self.window);
    }
  }

  fn process_open<P: AsRef<Path>>(&self, file_path: P) -> Result<(), OpenError> {
    let file_path = file_path.as_ref();
    let reader = OpenOptions::new().read(true).open(file_path).context(self::OpenFile { file_path })?;
    let calculator = GridCalculator::from_json(reader).context(self::OpenDeserialize { file_path })?;

    // PERF: setting Entries will trigger their signals, each which mutably borrow `state` and recalculates.

    self.gravity_multiplier.set(calculator.gravity_multiplier);
    self.container_multiplier.set(calculator.container_multiplier);
    self.planetary_influence.set(calculator.planetary_influence);
    self.additional_mass.set(calculator.additional_mass);
    self.ice_only_fill.set(calculator.ice_only_fill);
    self.ore_only_fill.set(calculator.ore_only_fill);
    self.any_fill_with_ice.set(calculator.any_fill_with_ice);
    self.any_fill_with_ore.set(calculator.any_fill_with_ore);
    self.any_fill_with_steel_plates.set(calculator.any_fill_with_steel_plates);
    {
      fn set_entries_from<'a>(entries: &HashMap<BlockId, Entry>, iter: impl Iterator<Item=(&'a BlockId, &'a u64)>) {
        for (block_id, count) in iter {
          if let Some(entry) = entries.get(block_id) {
            entry.set(count);
          }
        }
      }
      let block_entries = self.block_entries.borrow(); // Scoped borrow.
      for entry in block_entries.iter_entries() {
        entry.set("");
      }
      set_entries_from(&block_entries.entries, calculator.iter_block_counts());
      set_entries_from(&block_entries.up_entries, calculator.directional_blocks.get(&Direction::Up).unwrap().iter());
      set_entries_from(&block_entries.down_entries, calculator.directional_blocks.get(&Direction::Down).unwrap().iter());
      set_entries_from(&block_entries.front_entries, calculator.directional_blocks.get(&Direction::Front).unwrap().iter());
      set_entries_from(&block_entries.back_entries, calculator.directional_blocks.get(&Direction::Back).unwrap().iter());
      set_entries_from(&block_entries.left_entries, calculator.directional_blocks.get(&Direction::Left).unwrap().iter());
      set_entries_from(&block_entries.right_entries, calculator.directional_blocks.get(&Direction::Right).unwrap().iter());
    }

    let mut state = self.state.borrow_mut();
    state.current_file_path = Some(file_path.to_owned());
    state.current_dir_path = file_path.parent().map(|p| p.to_owned());
    state.calculator = calculator;
    Ok(())
  }

  fn save_or_save_as(&self) {
    let (current_dir_path, current_file_path) = {
      let state = self.state.borrow();
      (state.current_dir_path.clone(), state.current_file_path.clone())
    };
    if let Some(current_file_path) = current_file_path {
      self.process_save(current_file_path).show_error_as_dialog(&self.window);
    } else {
      let dialog = FileDialog::new_save(&self.window, current_dir_path, current_file_path);
      if let Some(file_path) = dialog.run() {
        self.process_save(file_path).show_error_as_dialog(&self.window);
      }
    }
  }

  fn save_as(&self) {
    let (current_dir_path, current_file_path) = {
      let state = self.state.borrow();
      (state.current_dir_path.clone(), state.current_file_path.clone())
    };
    let dialog = FileDialog::new_save(&self.window, current_dir_path, current_file_path);
    if let Some(file_path) = dialog.run() {
      self.process_save(file_path).show_error_as_dialog(&self.window);
    }
  }

  fn process_save<P: AsRef<Path>>(&self, file_path: P) -> Result<(), SaveError> {
    let file_path = file_path.as_ref();
    let writer = OpenOptions::new().write(true).create(true).open(file_path).context(self::SaveFile { file_path })?;

    let mut state = self.state.borrow_mut();
    state.calculator.to_json(writer).context(self::SaveSerialize { file_path })?;

    state.current_file_path = Some(file_path.to_owned());
    state.current_dir_path = file_path.parent().map(|p| p.to_owned());

    Ok(())
  }


  pub fn set_application(&self, app: &Application) {
    self.window.set_application(Some(app));
  }

  pub fn show(&self) {
    self.window.show_all();
  }
}


trait MyEntryExt: EntryExt {
  fn parse<T: FromStr + Copy>(&self, default: T) -> T;
  fn set<T: Display>(&self, value: T);

  fn insert_and_recalc_on_change<F: (Fn(&mut GridCalculator) -> &mut HashMap<BlockId, u64>) + 'static>(&self, main_window: &Rc<MainWindow>, id: BlockId, func: F);
  fn set_and_recalc_on_change<T: FromStr + Copy + 'static, F: (Fn(&mut GridCalculator) -> &mut T) + 'static>(&self, main_window: &Rc<MainWindow>, default: T, func: F);
}

impl MyEntryExt for Entry {
  fn parse<T: FromStr + Copy>(&self, default: T) -> T {
    self.get_text().map(|t| t.parse().unwrap_or(default)).unwrap_or(default)
  }

  fn set<T: Display>(&self, value: T) {
    self.set_text(&format!("{:.2}", value));
  }

  fn insert_and_recalc_on_change<F: (Fn(&mut GridCalculator) -> &mut HashMap<BlockId, u64>) + 'static>(&self, main_window: &Rc<MainWindow>, id: BlockId, func: F) {
    let rc_clone = main_window.clone();
    self.connect_changed(move |entry| {
      func(&mut rc_clone.state.borrow_mut().calculator).insert(id.clone(), entry.parse(0));
      rc_clone.recalculate();
    });
  }

  fn set_and_recalc_on_change<T: FromStr + Copy + 'static, F: (Fn(&mut GridCalculator) -> &mut T) + 'static>(&self, main_window: &Rc<MainWindow>, default: T, func: F) {
    let rc_clone = main_window.clone();
    self.connect_changed(move |entry| {
      *func(&mut rc_clone.state.borrow_mut().calculator) = entry.parse(default);
      rc_clone.recalculate();
    });
  }
}


trait MyLabelExt {
  fn set<T: Display>(&self, value: T);
}

impl MyLabelExt for Label {
  fn set<T: Display>(&self, value: T) {
    self.set_text(&format!("{:.2}", value));
  }
}