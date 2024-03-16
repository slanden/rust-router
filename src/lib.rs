/// # Features
/// * **eq-separator** -
///   Allow separating options from option-arguments
///   with a '='
/// * **single-hyphen-option-names** -
///   Changes options to expect a single "-" prefix
///   instead of "--", and short options are disabled
mod doc;
pub use {common::*, opt_map::optmap};
