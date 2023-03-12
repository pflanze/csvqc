/*
  Copyright by Christian Jaeger <ch@christianjaeger.ch>. Licensed under
  the GNU Public License version 3. Please contact me for other licenses
  or further work.
*/

pub use genawaiter::rc::Gen;
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

pub trait CellSettings {
    // Carries out the checks for a particular kind of cell, those in
    // addition to the standard checks.  For example, if the cell is
    // holding the name of a probe, then no number checks will be
    // carried out, whereas if it's a cell containing a number, this
    // will hold the check(s) for that particular number type,
    // e.g. SE, or average. cell_check_more must push the failures
    // onto `failures`.
    fn cell_check_more(&self,
                       cell: &CellContents,
                       failures: &mut Vec<CellCheckSubFailure>);
}

pub trait FileSettings {
    type CellSettingsT : CellSettings;
    // Determines what checks are carried out for a particular kind of
    // *file* (those in addition to the standard checks). Its role is
    // to provide a `CellSettings` depending on the row/column of a
    // cell. (Currently, only column is actually used, this might
    // change, depending on how we implement handling of headers.)

    // TODO: also add header description (for matching the right
    // FileSettings) or checks here.

    fn column_to_cellsettings(&self,
                              loc: &Location) -> Option<&Self::CellSettingsT>;
}

fn cell_check_empty(cell: &CellContents) -> Option<CellCheckSubFailure> {
    if cell == b"" {
        Some(CellCheckSubFailure { reason: String::from("empty cell") })
    } else {
        None
    }
}

fn cell_checks_whitespace(
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
fn cell_checks<CS: CellSettings>(
    cell: &CellContents,
    cellsettings: &CS,
    failures: &mut Vec<CellCheckSubFailure> // out
) {
    if let Some(f) = cell_check_empty(cell) {
        failures.push(f);
        return;
    }
    cell_checks_whitespace(cell, failures);
    if ! failures.is_empty() {
        return;
    }
    // Now come settings-dependent cell checks:
    cellsettings.cell_check_more(cell, failures);
}


// Parse the input as CSV and run the checks that run on cells
fn stream_checks_cells<R: io::Read, S: FileSettings>(
    input: R,
    settings: S
) -> impl Iterator<Item = Box<dyn CheckFailure>>
{
    let mut rb = csv::ReaderBuilder::new();
    rb.delimiter(b'\t');
    rb.has_headers(false); // XX actually treat it as such though
    let mut rdr = rb.from_reader(input);
    let mut record = csv::ByteRecord::new();
    let mut failures = Vec::new();
    let mut irow = 0;
    
    Gen::new(|co| async move {
        loop {
            match rdr.read_byte_record(&mut record) {
                Ok(true) => {
                    for (icell, cell) in record.iter().enumerate() {
                        let loc = Location { col: icell, row: irow };
                        if let Some(cellsettings) = settings.column_to_cellsettings(&loc) {
                            cell_checks(cell, cellsettings, &mut failures);
                        } else {
                            failures.push(
                                CellCheckSubFailure {
                                    reason: format!(
                                        "unexpected cell at location {}",
                                        loc)});
                        }
                        if ! failures.is_empty() {
                            co.yield_(
                                Box::new(
                                    CellCheckFailure {
                                        failures: failures.clone(),
                                        contents: Vec::from(cell),
                                        location: loc
                                    }) as Box<dyn CheckFailure>).await;
                            failures.clear();
                        }
                    }
                },
                Ok(false) => {
                    return;
                },
                Err(e) => {
                    co.yield_(
                        Box::new(
                            FileCheckFailure {
                                reason: format!("CSV parsing error: {}",
                                                e)
                            }) as Box<dyn CheckFailure>).await;
                    return;
                }
            }
            irow += 1;
        }
    }).into_iter()
}


fn file_checks_cells<F: FileSettings>(
    filepath: &Path,
    settings: F
) -> Result<impl Iterator<Item = Box<dyn CheckFailure>>> {
    let f = fs::File::open(filepath)?;
    Ok(stream_checks_cells(io::BufReader::new(f), settings).into_iter())
}

pub fn file_checks<F: FileSettings>(
    filepath: &Path,
    settings: F
) -> Result<impl Iterator<Item = Box<dyn CheckFailure>>> {
    // XX crlf part: TODO
    file_checks_cells(filepath, settings)
}

