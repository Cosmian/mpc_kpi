#![no_std]
#![no_main]
#![feature(min_const_generics)]

use scale_std::slice::Slice;

// Ranking of 3 players along these criteria
//  - crédit habitat
//  - crédit conso
//  - épargne LT
//  - épargne CT
//  - solde comptes courants
//
// this programs calculates for all criteria:
// - the sum and reveals it to all players
// - the ranking of each player and reveals it to that player
//
// The input data is expected to be rows of
// of the given criteria in that order rounded to the nearest Euro
// The first column is expected to represent tne month `YYYYMM` format
// e.g. 202102 for February 2021.
// The lines must be sorted by month in ascending order
//
// The output table for every player looks like
// Month, Sum criteria 1, Rank Criteria 1,...,Sum criteria 5, Rank Criteria 5

scale::main! {
    I64_MEMORY = 1000;
    SECRET_I64_MEMORY = 1000;
    SECRET_MODP_MEMORY = 1000;
    CLEAR_MODP_MEMORY = 1000;
    KAPPA = 40;
}
const CRITERIA: u64 = 5;

#[inline(never)]
fn main() {
    let end: SecretModp = SecretModp::from(ConstI32::<0>);
    print!("Program starting\n");
    // Read and process until an empty input is given.
    loop {
        print!("Reading next row of player 0...\n");
        // Read the next row for Player 0
        let row_0 = match read_next_record(Player::<0>, &Some(CRITERIA + 1)) {
            None => {
                break;
            }
            Some(s) => s,
        };
        let month = row_0.get(0);
        print!("...finding row player 1...\n");
        let row_1 = match find_record(Player::<1>, &month.into(), &Some(CRITERIA + 1)) {
            None => {
                print!("Stopping: month not found on player 1 !\n");
                break;
            }
            Some(s) => s,
        };
        print!("...finding row player 2...\n");
        let row_2 = match find_record(Player::<2>, &month.into(), &Some(CRITERIA + 1)) {
            None => {
                print!("Stopping: month not found on player 2 !\n");
                break;
            }
            Some(s) => s,
        };
        // Reveal the month to all players
        print!("...revealing month...\n");
        month.private_output(Player::<0>, Channel::<0>);
        month.private_output(Player::<1>, Channel::<0>);
        month.private_output(Player::<2>, Channel::<0>);
        // Process the 5 criteria
        for i in 1..(CRITERIA as u64) + 1 {
            // reveal the sum to all players
            let sum: ClearModp = (row_0.get(i) + row_2.get(i) + row_2.get(i)).reveal();
            print!("...revealing sum criteria...", i as i64, "\n");
            sum.output(Channel::<0>);
            // determine rankings
            let mut values: Slice<SecretI64> = Slice::uninitialized(3);
            values.set(0, &SecretI64::from(row_0.get(i)));
            values.set(1, &SecretI64::from(row_1.get(i)));
            values.set(2, &SecretI64::from(row_2.get(i)));
            let indexes = rescale(&sort(&values));
            // reveal the rankings selectively
            SecretModp::from(indexes.get(0)).private_output(Player::<0>, Channel::<0>);
            SecretModp::from(indexes.get(1)).private_output(Player::<1>, Channel::<0>);
            SecretModp::from(indexes.get(2)).private_output(Player::<2>, Channel::<0>);
            print!("...done criteria...", i as i64, "\n");
        }
        // signal end to all players
        end.private_output(Player::<0>, Channel::<1>);
        end.private_output(Player::<1>, Channel::<1>);
        end.private_output(Player::<2>, Channel::<1>);
        print!("... done row.\n");
    }
    print!("Done\n");
}

#[inline(always)]
fn find_record<const P: u32>(
    player: Player<P>,
    date: &SecretI64,
    expected_cols: &Option<u64>,
) -> Option<Slice<SecretModp>> {
    loop {
        match read_next_record(player, expected_cols) {
            None => {
                return None;
            }
            Some(s) => {
                let this_patient_id: SecretI64 = s.get(0).into();
                if this_patient_id.eq(*date).reveal() {
                    // found
                    return Some(s);
                }
                // loop
            }
        };
    }
}

#[inline(always)]
fn read_next_record<const P: u32>(
    player: Player<P>,
    expected_cols: &Option<u64>,
) -> Option<Slice<SecretModp>> {
    let num_cols = SecretModp::private_input(player, Channel::<0>).reveal();
    let num_cols = i64::from(num_cols) as u64;
    if num_cols == 0 {
        return None;
    }
    if let Some(n) = expected_cols {
        if num_cols != *n {
            print!(
                "ERROR: invalid number of columns: ",
                num_cols as i64, ", expected: ", *n as i64, "\n"
            );
            scale::assert!(num_cols == *n);
        }
    }
    let mut data = Slice::uninitialized(num_cols);
    for i in 0..num_cols {
        let _num_elt_cols = SecretModp::private_input(player, Channel::<1>); //no need to .reveal();
        data.set(i, &SecretModp::private_input(player, Channel::<2>));
    }
    Some(data)
}

#[inline(always)]
fn sort(values: &Slice<SecretI64>) -> Slice<SecretI64> {
    let n = values.len();
    let mut indexes: Slice<SecretI64> = Slice::uninitialized(n);
    for left in 0..n - 1 {
        for right in left + 1..n {
            let left_value = &values.get(left);
            let right_value = &values.get(right);
            let cmp = cmp(left_value, right_value);
            indexes.set(left, &(*left_value + cmp));
            indexes.set(right, &(*right_value - cmp));
        }
    }
    indexes
}

#[inline(always)]
fn cmp(a: &SecretI64, b: &SecretI64) -> SecretI64 {
    let cmp: SecretI64 = a.le(*b).into();
    -2 * cmp + 1
}

#[inline(always)]
fn rescale(indexes: &Slice<SecretI64>) -> Slice<SecretI64> {
    let n = indexes.len();
    let n_1 = SecretI64::from(n as i64 - 1);
    let mut rescaled: Slice<SecretI64> = Slice::uninitialized(n);
    for i in 0..n {
        let v = &indexes.get(i);
        rescaled.set(i, &((*v + n_1) >> ConstU32::<1>));
    }
    rescaled
}
