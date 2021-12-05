#![feature(is_sorted)]

use std::time::Duration;

use interruptable_function::prelude::*;

/// Sorting implemented using [Interruptable].
/// If the deadline is missed, the partially sorted data
/// is returned.
struct Sort<'a> {
    data: &'a mut [u8],
    idx: usize,
}

impl<'a> Sort<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        Self { data, idx: 0 }
    }

    /// Perform one pass of selection sort
    fn sorting_step(&mut self) {
        let min = self
            .data
            .iter()
            .skip(self.idx)
            .enumerate()
            .min_by_key(|(_, x)| *x)
            .map(|(i, _)| self.idx + i)
            .unwrap();

        self.data.swap(self.idx, min);
        self.idx += 1;
    }
}

impl<'a> Interruptable for Sort<'a> {
    type Output = Vec<u8>;

    fn poll(&mut self) -> Status<Self::Output> {
        self.sorting_step();
        if self.data.is_sorted() {
            Status::Done(self.data.to_vec())
        } else {
            Status::Pending
        }
    }

    /// The partial result of our sorting is just the data we are working on
    fn partial_result(&self) -> Option<Self::Output> {
        Some(self.data.to_vec())
    }
}

fn generate_data(len: usize) -> Vec<u8> {
    (0..len).map(|_| rand::random()).collect()
}

fn main() {
    let mut data = generate_data(1000);
    println!("{:?}", data.as_slice());

    let sort = Sort::new(&mut data);

    match exec_interruptable!(sort, Duration::from_millis(10)) {
        Ok(v) => println!("{:?}", v.as_slice()),
        Err(e) => {
            println!("{:?}", e.late_by());
            if let Some(partial) = e.partial_result() {
                println!("Partial result: {:?}", partial);
                let len = partial
                    .iter()
                    .zip(partial.iter().skip(1))
                    .take_while(|(p, q)| p <= q)
                    .count();
                println!("The first {} items were sorted", len);
            }
        }
    }
}
