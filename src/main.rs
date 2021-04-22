// Ranking of 3 participants along 5 criteria
//
// this programs calculates for all criteria:
// - the sum and reveals it to all participants
// - the ranking of each participant and reveals it to that player
//
// The input data is expected to be rows of
// of the given criteria in that order rounded to the nearest Euro
// The first column is expected to represent tne month `YYYYMM` format
// e.g. 202102 for February 2021.
// The lines must be sorted by month in ascending order
//
// The output table for every participant looks like
// Month, Value Criteria1, Sum criteria 1, Rank Criteria 1,...,Value Criteria5, Sum criteria 5, Rank Criteria 5

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use cosmian_std::{
    prelude::*,
    scale::{Reveal, ScaleCmp, SecretI64, SecretModp},
};
use cosmian_std::{scale, scale::println, InputRow, OutputRow};
use scale_std::slice::Slice;

// participants
const PARTICIPANT_0: Player<0> = Player::<0>;
const PARTICIPANT_1: Player<1> = Player::<1>;
const PARTICIPANT_2: Player<2> = Player::<2>;

const CRITERIA: u64 = 3;

#[cosmian_std::main(KAPPA = 40)]
#[inline(never)]
fn main() {
    println!("Program starting");
    // Read and process until an empty input is given.
    loop {
        println!("Reading next row of participant 0...");
        // Read the next row for participant 0
        let row_0 = match read_tabular(PARTICIPANT_0, CRITERIA + 1) {
            None => {
                //no more records => end of program
                println!("End of participant 0 rows");
                break;
            }
            Some(row) => row,
        };
        let month = *row_0.get_unchecked(0);
        println!("...finding row participant 1...");
        let row_1 = match find_tabular(PARTICIPANT_1, 0, &month, CRITERIA + 1) {
            None => {
                //Bob does no have this year => end of program
                println!("Stopping: month not found on participant 1 !");
                break;
            }
            Some(row) => row,
        };
        println!("...finding row participant 2...");
        let row_2 = match find_tabular(PARTICIPANT_2, 0, &month, CRITERIA + 1) {
            None => {
                //Bob does no have this year => end of program
                println!("Stopping: month not found on participant 1 !");
                break;
            }
            Some(row) => row,
        };
        // Prepare `OutputRow`s to reveal data to the participants
        let mut output_0 = OutputRow::new(PARTICIPANT_0);
        let mut output_1 = OutputRow::new(PARTICIPANT_1);
        let mut output_2 = OutputRow::new(PARTICIPANT_2);

        // Reveal the month to all participants
        println!("...revealing month...");
        output_0.append(month);
        output_1.append(month);
        output_2.append(month);

        // Process the 5 criteria
        for i in 1..(CRITERIA as u64) + 1 {
            println!("...processing criteria...", i as i64, "");
            let secret_value_0 = *row_0.get_unchecked(i);
            let secret_value_1 = *row_1.get_unchecked(i);
            let secret_value_2 = *row_2.get_unchecked(i);
            // "reveal" to each participant its own value
            output_0.append(secret_value_0);
            output_1.append(secret_value_1);
            output_2.append(secret_value_2);
            // reveal the sum to all participants
            let sum: SecretModp = secret_value_0 + secret_value_1 + secret_value_2;
            println!("...revealing sum criteria...", i as i64, "");
            output_0.append(sum);
            output_1.append(sum);
            output_2.append(sum);
            // determine rankings
            let mut secret_values: Slice<SecretI64> = Slice::uninitialized(3);
            secret_values.set(0, &SecretI64::from(secret_value_0));
            secret_values.set(1, &SecretI64::from(secret_value_1));
            secret_values.set(2, &SecretI64::from(secret_value_2));
            let ranks = secretly_rank(&secret_values, true);
            // reveal the rankings selectively
            println!("...revealing rankings criteria...", i as i64, "");
            output_0.append(SecretModp::from(*ranks.get_unchecked(0)));
            output_1.append(SecretModp::from(*ranks.get_unchecked(1)));
            output_2.append(SecretModp::from(*ranks.get_unchecked(2)));
            println!("...done criteria...", i as i64, "");
        }
        // signal end to all participants
        println!("... done row.");
    }
    println!("Done");
}

/// Read input data from a participant and expect it to be in a tabular format
/// with one scalar per column and a fixed number of columns
/// (e.g. a row of a CSV file)
fn read_tabular<const P: u32>(player: Player<P>, num_cols: u64) -> Option<Slice<SecretModp>> {
    let mut row = InputRow::read(player);
    println!("Num Cols: ", num_cols);
    let mut result = Slice::uninitialized(num_cols);
    for i in 0..num_cols {
        // println!("Reading col: ", i);
        let next_column = match row.next_col() {
            Some(c) => c,
            None => {
                //there is no more data to be read
                if i == 0 {
                    // this is the en of the dataset
                    return None;
                }
                // this will write the entry in the logs
                println!(
                    "ERROR: participant ",
                    P, ": invalid number of columns: ", num_cols, " expected but ", i, " found!"
                );
                scale::panic!(
                    "ERROR: participant ",
                    P,
                    ": invalid number of columns: ",
                    num_cols,
                    " expected but ",
                    i,
                    " found!"
                );
                // trick the compiler
                return None;
            }
        };
        // println!("Done Col: ", i);
        // we expect a single scalar value in the column
        let value = next_column
            .into_secret_modp()
            .expect("There should be a single scalar value in the column");
        result.set(i, &value);
    }
    Some(result)
}

/// Read tabular rows of data of a participant (see `read_tabular`)
/// Until it finds `value` in the column `column` number (starting from 0)
fn find_tabular<const P: u32>(
    player: Player<P>,
    column: u64,
    value: &SecretModp,
    num_cols: u64,
) -> Option<Slice<SecretModp>> {
    // secret comparisons are performed on boolean circuits
    // so we need to switch to another representation for our value
    let value_f2: SecretI64 = (*value).into();
    loop {
        match read_tabular(player, num_cols) {
            None => {
                return None;
            }
            Some(row) => {
                // convert the value read, as above
                let this_value: SecretI64 = (*row.get_unchecked(column)).into();
                // we need to reveal the equality to test it
                // in clear text with an if
                if this_value.eq(value_f2).reveal() {
                    // ok found
                    return Some(row);
                }
                // not found =>_loop
            }
        };
    }
}

/// Rank the values from 1..n provided in tre `values` slice of size `n`.
/// The output slice will contain the rank of the corresponding value in the
/// `values` input slice i.e input values of:
///
///  -  Secret([11, 33, 22]) will output Secret([1, 3, 2 ]) ascending
///  -  Secret([11, 33, 22]) will output Secret([3, 1, 2 ]) descending
///
/// The algorithm is designed in such a way all data stay secret
/// during the processing and nothing is revealed
fn secretly_rank(values: &Slice<SecretI64>, descending: bool) -> Slice<SecretI64> {
    let n = values.len();
    let mut ranks: Slice<SecretI64> = Slice::uninitialized(n);
    for left in 0..n - 1 {
        for right in left + 1..n {
            let left_value = &*values.get_unchecked(left);
            let right_value = &*values.get_unchecked(right);
            let cmp = cmp(left_value, right_value);
            if descending {
                ranks.set(left, &(*ranks.get_unchecked(left) - cmp));
                ranks.set(right, &(*ranks.get_unchecked(right) + cmp));
            } else {
                ranks.set(left, &(*ranks.get_unchecked(left) + cmp));
                ranks.set(right, &(*ranks.get_unchecked(right) - cmp));
            }
        }
    }
    rescale(&ranks)
}

/// Secretly compare 2 secret values.
///
/// This function returns a **secret**
///
/// - Secret(-1) if a <= b
/// - Secret(1) otherwise
#[inline(always)]
fn cmp(a: &SecretI64, b: &SecretI64) -> SecretI64 {
    let cmp: SecretI64 = a.le(*b).into();
    -2 * cmp + 1
}

/// Used by the `rank` function to rescale the ranks before output.
///
/// The base arithmetic algorithm will rank 3 values outputting [-2,0,2],
/// This rescales the ranks to [1,2,3]
fn rescale(indexes: &Slice<SecretI64>) -> Slice<SecretI64> {
    let n = indexes.len();
    let n_1 = SecretI64::from(n as i64 - 1);
    let mut rescaled: Slice<SecretI64> = Slice::uninitialized(n);
    for i in 0..n {
        let v = *indexes
            .get(i)
            .expect("there should be an index at that position");
        rescaled.set(i, &(((v + n_1) >> ConstU32::<1>) + 1));
    }
    rescaled
}
