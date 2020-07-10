use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::style::{Color, Print};
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, terminal};
use futures::Future;
use std::io::{stderr, Write};
use termimad::{CompoundStyle, MadSkin};
use tokio::sync::{
    oneshot,
    oneshot::{error::TryRecvError, Sender},
};
use tokio::task::JoinHandle;
use tokio::time;

use crate::error::Result;

const LOADING_SPINNER_DELAY: u64 = 40;
const LOADING_SPINNER_DOTS: [&str; 56] = [
    "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉",
    "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠",
    "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩",
    "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀",
];

pub struct Term {
    skin: MadSkin,
}

impl Default for Term {
    fn default() -> Self {
        Term::new()
    }
}

impl Term {
    pub fn new() -> Self {
        let mut skin = MadSkin::default();
        skin.inline_code = CompoundStyle::with_fg(Color::Cyan);
        skin.code_block.compound_style = CompoundStyle::with_fg(Color::Cyan);
        Term { skin }
    }

    /// Print text to stdout
    pub fn print(&self, text: &str) {
        self.skin.print_text(text)
    }

    /// Print text with error styling to stderr
    /// Needs mut to temporarily modify styling (e.g. red fg)
    pub fn print_error(&mut self, text: &str) -> Result<()> {
        self.print_with_style(Color::Red, "✖ ", text)
    }

    /// Print text with notice styling to stderr
    /// Needs mut to temporarily modify styling (e.g. yellow fg)
    pub fn print_notice(&mut self, text: &str) -> Result<()> {
        self.print_with_style(Color::Yellow, "➜ ", text)
    }

    fn print_with_style(&mut self, fg: Color, prefix: &str, text: &str) -> Result<()> {
        let mut styled_text = String::from(prefix);
        styled_text.push_str(text);
        // Set fg
        self.skin.paragraph.set_fg(fg);
        self.skin
            .write_text_on(&mut std::io::stderr(), &styled_text)?;
        // Unset fg
        self.skin
            .paragraph
            .compound_style
            .object_style
            .foreground_color = None;
        Ok(())
    }

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
        let (tx, spinner_handle) = Self::spinner();

        let result = future.await;

        // Stop spinner
        tx.send(()).ok();
        spinner_handle.await??;

        Ok(result)
    }

    /// Start a CLI spinner on the current cursor line. To stop it, call `send` on the `Sender`. To
    /// wait until it's done cleaning up it's current action (which is very important), await it's
    /// `JoinHandle`.
    fn spinner() -> (Sender<()>, JoinHandle<Result<()>>) {
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
            while let Err(TryRecvError::Empty) = rx.try_recv() {
                execute!(
                    stderr(),
                    cursor::MoveToColumn(0),
                    terminal::Clear(ClearType::CurrentLine),
                    Print(dots.next().unwrap())
                )?;
                interval.tick().await;
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
}

pub fn print_error(text: &str) -> Result<()> {
    Term::new().print_error(text)
}
