#[macro_export]
macro_rules! printerr {
    ($($arg:tt)*) => ({
        use std::io::{Write, stderr};
        use crossterm::{execute};
        use crossterm::style::{Print, SetForegroundColor, ResetColor, Color};
        execute!(
            stderr(),
            SetForegroundColor(Color::Red),
            Print("✖ ".to_string()),
            Print($($arg)*.to_string()),
            ResetColor
        ).ok();
    })
}
