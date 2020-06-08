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

#[macro_export]
macro_rules! print_error {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Red);
        termimad::mad_write_inline!(err, $skin, "✖ ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_notice {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Yellow);
        termimad::mad_write_inline!(err, $skin, "✖ ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}
