/*
  Copyright by Christian Jaeger <ch@christianjaeger.ch>. Licensed under
  the GNU Public License version 3. Please contact me for other licenses
  or further work.
 */

mod check;
use anyhow::{Result, Context, bail};
use check::{CellContents, FileSettings, CellSettings}; 
use check::checkfailure::{location::Location, CellCheckSubFailure};
use std::path::PathBuf;
use bstr_parse::BStrParse;
use regex::bytes::Regex;
use lazy_static::lazy_static;
use clap::Parser;


// Check that the cell consists of just a natural number in the u32 range
fn cell_check_id_u32(
    cell: &CellContents,
    failures: &mut Vec<CellCheckSubFailure>
) {
    match cell.parse::<u32>() {
        Ok(n) => {
            if n < 1 {
                failures.push(CellCheckSubFailure {
                    reason: format!("cell_check_id_u32: IDs must be natural numbers, \
                                     starting from 1; got {}", n) });
            }
        },
        Err(e) => {
            failures.push(CellCheckSubFailure {
                reason: format!("cell_check_id_u32: {}", e) });
        }
    }
}

#[allow(non_snake_case)]
fn cell_is_NA(cell: &CellContents) -> bool {
    cell == b"\\N"
}

static CELL_CHECK_BXNAME_RE : &'static str = r"^(?:\d+|/\w+|[A-Za-z_<.>()-]+)+$";

fn cell_check_bxname(
    cell: &CellContents,
    failures: &mut Vec<CellCheckSubFailure>
) {
    lazy_static! {
        static ref RE: Regex = Regex::new(CELL_CHECK_BXNAME_RE).unwrap();
    }
    if ! RE.is_match(cell) {
        failures.push(CellCheckSubFailure {
            reason: format!("cell_check_bxname: not matching {}",
                            CELL_CHECK_BXNAME_RE)});
    }
}

fn cell_check_optional_bxname(
    cell: &CellContents,
    failures: &mut Vec<CellCheckSubFailure>
) {
    if ! cell_is_NA(cell) {
        cell_check_bxname(cell, failures)
    }
}

fn cell_check_symbol_or_alias(
    _cell: &CellContents,
    _failures: &mut Vec<CellCheckSubFailure>
) {
    // ... whatever ...   
}

const STRAIN_FILE_CELLCHECKER_BY_COL : [
    fn(&CellContents, &mut Vec<CellCheckSubFailure>); 6
] = [
    cell_check_id_u32, // Id
    cell_check_bxname, // Name
    cell_check_optional_bxname, // Name2
    cell_check_id_u32, // SpeciesId
    cell_check_symbol_or_alias, // Symbol
    cell_check_symbol_or_alias, // Alias
];

struct CellCheckerFn(fn(&CellContents, &mut Vec<CellCheckSubFailure>));

impl CellSettings for CellCheckerFn {
    fn cell_check_more(&self,
                       cell: &CellContents,
                       failures: &mut Vec<CellCheckSubFailure>) {
        (self.0)(cell, failures);
    }
}

struct FileSettingsByCol(Vec<CellCheckerFn>);

impl FileSettings for FileSettingsByCol {
    type CellSettingsT = CellCheckerFn;
    fn location_to_cellsettings(&self, loc: &Location) -> Option<&CellCheckerFn> {
        self.0.get(loc.col)
    }
}


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// Path to the file to be checked
   #[clap(value_parser, required(true))]
   file_path: PathBuf,
}


fn main() -> Result<()> {
    let args = Args::parse();
    
    let cellsettings_by_col = STRAIN_FILE_CELLCHECKER_BY_COL.iter().map(
        |chk| { CellCheckerFn(*chk)}).collect::<Vec<_>>();
    
    let failures = check::file_checks(
        &args.file_path,
        FileSettingsByCol(cellsettings_by_col)
    ).with_context(|| format!("can't check file {:?}", args.file_path))?;

    let mut num_failures : u64 = 0;
    for failure in failures {
        num_failures += 1;
        println!("Failure: {}", failure.plaintext_message());
    }

    if num_failures > 0 {
        bail!("found {} check violations", num_failures);
    }
        
    Ok(())
}
