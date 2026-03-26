pub mod colors;
pub mod components;
#[macro_use]
pub mod macros;
pub mod theme;
pub mod utils;

pub fn init() {
  yansi::whenever(yansi::Condition::TTY_AND_COLOR);
}
