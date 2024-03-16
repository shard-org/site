use std::fmt::Display;

pub trait HandleErr<T> {
    fn handle(self) -> T;
}

impl<E: Display, T> HandleErr<T> for Result<T, E> {
    fn handle(self) -> T {
        self.unwrap_or_else(|e| {
            println!("\x1b[31;1mERR:\x1b[0m {}", e);
            std::process::exit(1)
        })
    }
}


pub trait MapIf {
    fn map_if<F>(self, f: F) -> Option<Self> 
    where Self: Sized, F: FnOnce(&Self) -> bool;
}

impl<T> MapIf for T {
    fn map_if<F: FnOnce(&T) -> bool>(self, f: F) -> Option<Self> {
        if f(&self) { 
            return Some(self);
        } None
    }
}
