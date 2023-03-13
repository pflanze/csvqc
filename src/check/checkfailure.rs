/*
  Copyright by Christian Jaeger <ch@christianjaeger.ch>. Licensed under
  the GNU Public License version 3. Please contact me for other licenses
  or further work.
*/

pub mod location;

pub trait CheckFailure {
    fn reason(&self) -> String;
    fn plaintext_message(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct FileCheckFailure {
    pub reason: String,
}

impl CheckFailure for FileCheckFailure {
    fn reason(&self) -> String {
        self.reason.clone()
    }
    fn plaintext_message(&self) -> String {
        self.reason()
    }
}

// A failure for a single cell test
#[derive(Clone, Debug)]
pub struct CellCheckSubFailure {
    pub reason: String,
}

// All failures for one cell
#[derive(Clone, Debug)]
pub struct CellCheckFailure {
    pub failures: Vec<CellCheckSubFailure>,
    pub contents: Vec<u8>,
    pub location: location::Location
}

impl CheckFailure for CellCheckFailure {
    fn reason(&self) -> String {
        let mut s = String::new();
        for failure in self.failures.iter() {
            s.push_str(&failure.reason);
            s.push('\n');
        }
        s
    }
    fn plaintext_message(&self) -> String {
        let mut s = self.reason();
        s.push_str(&format!("  in cell {}", self.location));
        s.push('\n');
        match std::str::from_utf8(&self.contents) {
            Ok(sr) =>
                s.push_str(&format!("  contents: {:?}", sr)),
            Err(_) =>
                s.push_str(&format!("  contents: {:?}", self.contents))
        }
        s
    }
}

