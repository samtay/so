use crate::error::{Error, Result};
use crossterm::style::Color;
use lazy_static::lazy_static;
use minimad::mad_inline;
use std::io::Stderr;
use termimad::{mad_write_inline, MadSkin};

pub fn with_error_style<R, F>(skin: &mut MadSkin, f: F) -> Result<R>
where
    F: FnOnce(&MadSkin, &mut Stderr) -> Result<R, termimad::Error>,
{
    let err = &mut std::io::stderr();
    let p = skin.paragraph.clone();
    skin.paragraph.set_fg(Color::Red);
    mad_write_inline!(err, skin, "✖ ")?;
    let r: R = f(&skin, err)?;
    skin.paragraph = p;
    Ok::<R, Error>(r)
}

/// This makes code much more convenient, but would require each style to own
/// its own skin clone. Not sure if it is worth it.
pub fn mk_print_error(skin: &MadSkin) -> impl FnMut(&str) -> Result<()> + 'static {
    let mut skin = skin.clone();
    move |text: &str| {
        with_error_style(&mut skin, |err_skin, stderr| {
            err_skin.write_text_on(stderr, text)
        })
    }
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
        termimad::mad_write_inline!(err, $skin, "➜ ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_success {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Green);
        termimad::mad_write_inline!(err, $skin, "✔ ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_log {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Blue);
        termimad::mad_write_inline!(err, $skin, "• ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_warn {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Magenta);
        termimad::mad_write_inline!(err, $skin, "⚡").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}
