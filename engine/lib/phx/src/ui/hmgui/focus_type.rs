// TODO: do we really need this mouse/scroll 'focus' separation?

/// Specify the purpose of mouse over check:
/// - Mouse: regular mouse over element check
/// - Scroll: use to control mouse over scroll area
///
/// Two types to avoid collision between scrolling and mouse over element check.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FocusType {
    Mouse,
    Scroll,
}
