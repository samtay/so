use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::style::{Color, Print};
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, terminal};
use futures::Future;
use lazy_static::lazy_static;
use minimad::mad_inline;
use std::io::{stderr, Stderr, Write};
use termimad::{mad_write_inline, MadSkin};
use tokio::sync::{
    oneshot,
    oneshot::{error::TryRecvError, Sender},
};
use tokio::task::JoinHandle;
use tokio::time;

use crate::error::{Error, Result};

const LOADING_SPINNER_DELAY: u64 = 40;
const LOADING_SPINNER_DOTS: [&str; 56] = [
    "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉",
    "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠",
    "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩",
    "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀",
];

/// Blocks and waits for the user to press any key. Returns whether or not that key is the
/// character key `c`.
pub fn wait_for_char(c: char) -> Result<bool> {
    let mut pressed = false;
    terminal::enable_raw_mode()?;
    loop {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                ..
            }) if ch == c => {
                pressed = true;
                break;
            }
            Event::Key(_) => break,
            _ => (),
        }
    }
    terminal::disable_raw_mode()?;
    Ok(pressed)
}

/// As it sounds, takes a future and shows a CLI spinner until it's output is ready
pub async fn wrap_spinner<F>(future: F) -> Result<F::Output>
where
    F: Future,
{
    // Start spinner
    let (tx, spinner_handle) = spinner();

    let result = future.await;

    // Stop spinner
    tx.send(()).ok();
    spinner_handle.await??;

    Ok(result)
}

/// Start a CLI spinner on the current cursor line. To stop it, call `send` on the `Sender`. To
/// wait until it's done cleaning up it's current action (which is very important), await it's
/// `JoinHandle`.
pub fn spinner() -> (Sender<()>, JoinHandle<Result<()>>) {
    let (tx, mut rx) = oneshot::channel();
    let spinner_handle = tokio::spawn(async move {
        let mut dots = LOADING_SPINNER_DOTS.iter().cycle();
        terminal::enable_raw_mode()?;
        execute!(
            stderr(),
            cursor::SavePosition,
            cursor::Hide,
            terminal::Clear(ClearType::CurrentLine),
        )?;
        let mut interval = time::interval(time::Duration::from_millis(LOADING_SPINNER_DELAY));
        loop {
            match rx.try_recv() {
                Err(TryRecvError::Empty) => {
                    execute!(
                        stderr(),
                        cursor::MoveToColumn(0),
                        terminal::Clear(ClearType::CurrentLine),
                        Print(dots.next().unwrap())
                    )?;
                    interval.tick().await;
                }
                _ => break,
            }
        }
        execute!(
            stderr(),
            terminal::Clear(ClearType::CurrentLine),
            cursor::RestorePosition,
            cursor::Show,
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    });
    (tx, spinner_handle)
}

/// Temporarily modifies a skin with error styles (e.g. red fg) for use with the given closure.
/// Once the closure finishes, the skin is returned to original state.
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
