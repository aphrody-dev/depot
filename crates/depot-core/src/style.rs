/// Lightweight ANSI VT styling trait replacing colored crate.
pub trait StyleExt {
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn cyan(&self) -> String;
    fn magenta(&self) -> String;
    fn bold(&self) -> String;
}

impl<T: std::fmt::Display + ?Sized> StyleExt for T {
    fn red(&self) -> String {
        format!("\x1b[31m{}\x1b[0m", self)
    }
    fn green(&self) -> String {
        format!("\x1b[32m{}\x1b[0m", self)
    }
    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }
    fn cyan(&self) -> String {
        format!("\x1b[36m{}\x1b[0m", self)
    }
    fn magenta(&self) -> String {
        format!("\x1b[35m{}\x1b[0m", self)
    }
    fn bold(&self) -> String {
        format!("\x1b[1m{}\x1b[0m", self)
    }
}
