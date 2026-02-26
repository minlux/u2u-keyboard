// ---------------------------------------------------------------------------
// Compile-time keyboard layout selection
//
// Exactly one of the following Cargo features must be enabled:
//   layout-qwerty   US/UK QWERTY
//   layout-qwertz   German/Austrian QWERTZ
//
// Each layout file implements `ascii_to_hid`, the single public function
// exported from this module.  Adding a new layout means:
//   1. Create src/layout_<name>.rs with ascii_to_hid()
//   2. Add a feature in Cargo.toml
//   3. Add a #[cfg] / #[path] pair below
// ---------------------------------------------------------------------------

#[cfg(all(feature = "layout-qwerty", feature = "layout-qwertz"))]
compile_error!("Only one keyboard layout feature may be active at a time \
                (layout-qwerty or layout-qwertz).");

#[cfg(not(any(feature = "layout-qwerty", feature = "layout-qwertz")))]
compile_error!("A keyboard layout feature must be enabled: \
                layout-qwerty or layout-qwertz.");

#[cfg(feature = "layout-qwerty")]
#[path = "layout_qwerty.rs"]
mod active;

#[cfg(feature = "layout-qwertz")]
#[path = "layout_qwertz.rs"]
mod active;

pub use active::ascii_to_hid;
