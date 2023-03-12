/*
  Copyright by Christian Jaeger <ch@christianjaeger.ch>. Licensed under
  the GNU Public License version 3. Please contact me for other licenses
  or further work.
*/

pub use genawaiter::rc::gen;
pub use genawaiter::*;
pub use ::genawaiter::*;
pub use ::genawaiter::rc_producer;
pub mod checkfailure;
use anyhow::Result; 
use std::{fs, io};
use std::iter::Iterator;
use std::path::Path;
use std::str;
use csv;

use self::checkfailure::{CheckFailure, FileCheckFailure,
                         CellCheckFailure, CellCheckSubFailure};
use self::checkfailure::location::Location;

#[allow(dead_code)]
pub type CellContents = [u8];

#[allow(dead_code)]
pub struct CellSettings {
    // Determines what checks are carried out for a particular kind of
    // *cell*, in addition to the standard checks.  For example, if the
    // cell is holding the name of a probe, then no number checks will be
    // carried out, whereas if it's a cell containing a number, this will
    // hold the check(s) for that particular number type, e.g. SE, or
    // average.

    // Examples:
    //    CellSettings(cell_check_ProbeSet)
    //    CellSettings(cell_check_proper_average)
    pub cell_check_more: Box<dyn Fn(&CellContents,
                                    &mut Vec<CellCheckSubFailure>)>
}

#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct FileSettings {
    // Determines what checks are carried out for a particular kind of
    // *file*, in addition to the standard checks. Its role is to provide
    // `CellSettings` depending on the row/column of a cell. (Currently,
    // only column is actually used, this might change, depending on how
    // we implement handling of headers.)

    // FUTURE: header description
    // ?
    pub column_to_CellSettings: Box<dyn Fn(&Location)-> CellSettings>
}

fn _cell_check_empty(cell: &CellContents) -> Option<CellCheckSubFailure> {
    if cell == b"" {
        Some(CellCheckSubFailure { reason: String::from("empty cell") })
    } else {
        None
    }
}

fn _cell_checks_whitespace(
    cell: &CellContents,
    failures: &mut Vec<CellCheckSubFailure> // out
) {
    if cell.is_empty() {
        return; // should never happen
    }
    for (i, b) in cell.iter().enumerate() {
        if b.is_ascii_whitespace() {
            let reason =
                if i == 0 {
                    "leading whitespace"
                } else if i == cell.len() - 1 {
                    "trailing whitespace"
                } else {
                    "whitespace in the middle"
                };
            failures.push(CellCheckSubFailure {
                reason: String::from(reason)
            });
            return;
        } else if *b >= 128u8 {
            // Decode the remainder
            match str::from_utf8(&cell[i..]) {
                Ok(rem) => {
                    for (i2, c) in rem.chars().enumerate() {
                        if c.is_whitespace() {
                            failures.push(CellCheckSubFailure {
                                reason: format!("whitespace (in cell with unicode) at pos {}", i+i2)
                            });
                            return;
                        }
                    }
                },
                Err(e) => {
                     failures.push(CellCheckSubFailure {
                         reason: format!("UTF-8 decoding error: {}", e)
                     })
                }
            }
        }
    }
}

// Main function for all cell checks.
fn _cell_checks(
    cell: &CellContents,
    cellsettings: &CellSettings,
    failures: &mut Vec<CellCheckSubFailure> // out
) {
    if let Some(f) = _cell_check_empty(cell) {
        failures.push(f);
        return;
    }
    _cell_checks_whitespace(cell, failures);
    if ! failures.is_empty() {
        return;
    }
    // Now come settings-dependent cell checks:
    (cellsettings.cell_check_more)(cell, failures);
}


// Parse the input as CSV and run the checks that run on cells
fn stream_checks_cells<R: io::Read>(
    input: R,
    settings: FileSettings
) -> impl Iterator<Item = Box<dyn CheckFailure>>
{
    let mut rb = csv::ReaderBuilder::new();
    rb.delimiter(b'\t');
    rb.has_headers(false); // XX actually treat it as such though
    let mut rdr = rb.from_reader(input);
    let mut record = csv::ByteRecord::new();
    let mut failures = Vec::new();
    let mut irow = 0;
    
    gen!({
        loop {
            match rdr.read_byte_record(&mut record) {
                Ok(true) => {
                    for (icell, cell) in record.iter().enumerate() {
                        let loc = Location { col: icell, row: irow };
                        let cellsettings =
                            (settings.column_to_CellSettings)(&loc);
                        _cell_checks(cell, &cellsettings, &mut failures);
                        if ! failures.is_empty() {
                            yield_!(
                                Box::new(
                                    CellCheckFailure {
                                        failures: failures.clone(),
                                        contents: Vec::from(cell),
                                        location: loc
                                    }) as Box<dyn CheckFailure>);
                            failures.clear();
                        }
                    }
                },
                Ok(false) => {
                    return;
                },
                Err(e) => {
                    yield_!(
                        Box::new(
                            FileCheckFailure {
                                reason: format!("CSV parsing error: {}",
                                                e)
                            }) as Box<dyn CheckFailure>);
                    return;
                }
            }
            irow += 1;
        }
    }).into_iter()
}


fn file_checks_cells(
    filepath: &Path,
    settings: FileSettings
) -> Result<impl Iterator<Item = Box<dyn CheckFailure>>> {
    let f = fs::File::open(filepath)?;
    Ok(stream_checks_cells(io::BufReader::new(f), settings).into_iter())
}

pub fn file_checks(
    filepath: &Path,
    settings: FileSettings
) -> Result<impl Iterator<Item = Box<dyn CheckFailure>>> {
    // XX crlf part: TODO
    file_checks_cells(filepath, settings)
}

