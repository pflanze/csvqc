/*
  Copyright by Christian Jaeger <ch@christianjaeger.ch>. Licensed under
  the GNU Public License version 3. Please contact me for other licenses
  or further work.
 */

mod check;
use anyhow::Result;
use check::{CellContents, FileSettings, CellSettings}; 
use check::checkfailure::{location::Location, CellCheckSubFailure};
use std::path::PathBuf;
use bstr_parse::{BStrParse, ParseIntError, FromBStr};

// Check that the cell consists of just a natural number in the s32 range
fn cell_check_id_s32(
    _cell: &CellContents,
    _failures: &mut Vec<CellCheckSubFailure>
) {
    
}

fn cell_check_bxname(
    _cell: &CellContents,
    _failures: &mut Vec<CellCheckSubFailure>
) {
    
}

fn cell_check_symbol_or_alias(
    _cell: &CellContents,
    _failures: &mut Vec<CellCheckSubFailure>
) {
    
}

const STRAIN_FILE_CELLCHECKER_BY_COL : [fn(&CellContents, &mut Vec<CellCheckSubFailure>); 6] = [
    cell_check_id_s32, // Id
    cell_check_bxname, // Name
    cell_check_bxname, // Name2
    cell_check_id_s32, // SpeciesId
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
    fn column_to_cellsettings(&self, loc: &Location) -> Option<&CellCheckerFn> {
        self.0.get(loc.col)
    }
}


fn main() -> Result<()> {
    let cellsettings_by_col = STRAIN_FILE_CELLCHECKER_BY_COL.iter().map(
        |chk| { CellCheckerFn(*chk)}).collect::<Vec<_>>();
    
    let failures = check::file_checks(
        &PathBuf::from("/home/chrisrust/tmp/Strain_062521_headers.csv"),
        FileSettingsByCol(cellsettings_by_col)
    )?;
    for failure in failures {
        println!("Failure: {}", failure.plaintext_message());
    }
    Ok(())
}
