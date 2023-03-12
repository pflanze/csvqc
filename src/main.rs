/*
  Copyright by Christian Jaeger <ch@christianjaeger.ch>. Licensed under
  the GNU Public License version 3. Please contact me for other licenses
  or further work.
 */

#[macro_use]
extern crate genawaiter;

mod check;
use anyhow::Result;
use check::{CellContents, FileSettings, CellSettings}; 
use check::checkfailure::{location::Location, CellCheckSubFailure};
use std::path::PathBuf;

fn cell_check_more_1(
    _cell: &CellContents,
    _failures: &mut Vec<CellCheckSubFailure>
) {
    // XX add some...
}

#[allow(non_snake_case)]
fn column_to_CellSettings_1(_loc: &Location) -> CellSettings {
    CellSettings {
        cell_check_more: Box::new(cell_check_more_1)
    }
}


fn main() -> Result<()> {
    let failures = check::file_checks(
        &PathBuf::from("/home/chrisrust/tmp/Strain_062521_headers.csv"),
        FileSettings { column_to_CellSettings:
                       Box::new(column_to_CellSettings_1) }
    )?;
    for failure in failures {
        println!("Failure: {}", failure.plaintext_message());
    }
    Ok(())
}
