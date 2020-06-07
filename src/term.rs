use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::QueueableCommand;
use std::io::{Stderr, Write};

pub trait ColoredOutput {
    fn queue_general(&mut self, color: Color, prefix: &str, s: &str) -> &mut Self;

    // TODO is it cool to unwrap flushing some known text?
    fn unsafe_flush(&mut self);

    fn queue_error(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Red, "✖ ", s)
    }

    fn queue_success(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Green, "✔ ", s)
    }

    fn queue_notice(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Yellow, "➜ ", s)
    }

    fn queue_notice_inline(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Yellow, "", s)
    }

    fn queue_log(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Blue, "• ", s)
    }

    fn queue_code(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Cyan, "\t", s)
    }

    fn queue_code_inline(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Cyan, "", s)
    }

    fn queue_warn(&mut self, s: &str) -> &mut Self {
        self.queue_general(Color::Magenta, "⚡", s)
    }
}

impl ColoredOutput for Stderr {
    fn queue_general(&mut self, color: Color, prefix: &str, s: &str) -> &mut Self {
        (|| -> Result<(), crossterm::ErrorKind> {
            self.queue(SetForegroundColor(color))?
                .queue(Print(prefix.to_string()))?
                .queue(Print(s.to_string()))?
                .queue(ResetColor)?;
            Ok(())
        })()
        .unwrap();
        self
    }

    fn unsafe_flush(&mut self) {
        self.flush().unwrap();
    }
}
