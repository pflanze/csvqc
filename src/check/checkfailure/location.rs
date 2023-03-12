/*
  Copyright by Christian Jaeger <ch@christianjaeger.ch>. Licensed under
  the GNU Public License version 3. Please contact me for other licenses
  or further work.
*/

use std::fmt;

#[derive(Clone, Debug)]
pub struct Location {
    pub row: u64,
    pub col: usize
}

fn col_to_char(c: usize) -> char {
    (c + ('A' as usize)) as u8 as char
}

fn col_to_string(col: usize) -> String {
    fn recur(c: usize, s: &mut String, is_first_call: bool) {
        if c >= 26 {
            recur(c / 26, s, false);
        }
        let r = c % 26;
        s.push(col_to_char(if is_first_call { r } else { r - 1 }));
    }

    let mut s = String::new();
    recur(col, &mut s, true);
    s
}

impl fmt::Display for Location {
    // Use Excel `A1` style location format
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", col_to_string(self.col), self.row + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t0() {
        assert_eq!(format!("{}", Location { row: 0, col: 0 }),   "A1");
    }
    #[test]
    fn t1() {
        assert_eq!(format!("{}", Location { row: 99, col: 1 }), "B100");
    }
    #[test]
    fn t_last(){
        assert_eq!(format!("{}", Location { row: 0, col: 25 }),  "Z1");
    }
    #[test]
    fn t_wrap() {
        assert_eq!(format!("{}", Location { row: 0, col: 26 }),  "AA1");
    }
    #[test]
    fn t_correct_order_of_digits() {
        assert_eq!(format!("{}", Location { row: 0, col: 27 }),  "AB1");
    }
    #[test]
    fn t_last_2() {
        assert_eq!(format!("{}", Location { row: 0, col: 51 }),  "AZ1");
    }
    #[test]
    fn t_wrap_2() {
        assert_eq!(format!("{}", Location { row: 0, col: 52 }),  "BA1");
    }
    #[test]
    fn t_1022() {
        assert_eq!(format!("{}", Location { row: 0, col: 1022 }),  "AMI1");
    }
}
