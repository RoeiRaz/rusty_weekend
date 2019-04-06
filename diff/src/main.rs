extern crate unicode_segmentation;

use core::fmt::Write;
use std::cmp::min;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Error, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Display;
use std::vec::Vec;

use unicode_segmentation::UnicodeSegmentation;

trait DiffGraphable: Debug + Eq + Default {}

impl<T: Debug + Eq + Default> DiffGraphable for T {}

struct DiffGraph<T> where T: DiffGraphable {
    orig_lines: Vec<T>,
    target_lines: Vec<T>,
}

impl<T> DiffGraph<T> where T: DiffGraphable {
    fn new(orig_lines: Vec<T>, target_lines: Vec<T>) -> DiffGraph<T> {
        DiffGraph {
            orig_lines,
            target_lines,
        }
    }

    fn original_length(&self) -> usize { self.orig_lines.len() }
    fn target_length(&self) -> usize { self.target_lines.len() }

    fn skippable(&self, point: DiffGraphPoint) -> bool {
        let next_pair = (self.target_lines.get(point.i), self.orig_lines.get(point.j));
        match next_pair {
            (Some(target), Some(orig)) => target == orig,
            _ => false,
        }
    }

    fn snake_descent(&self, from: &DiffGraphPoint) -> DiffGraphPoint {
        let mut ret: DiffGraphPoint = from.clone();
        while self.skippable(ret) { ret = ret.delta_clone(1, 1); }
        ret
    }
}

#[derive(Copy, Clone, Debug)]
struct DiffGraphPoint { i: usize, j: usize }

impl DiffGraphPoint {
    fn delta_clone(&self, di: usize, dj: usize) -> Self {
        Self { i: self.i + di, j: self.j + dj }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
struct DiffGraphPathParameters { d: usize, k: usize }

impl From<(usize, usize)> for DiffGraphPoint {
    fn from(tuple: (usize, usize)) -> Self {
        Self { i: tuple.0, j: tuple.1 }
    }
}

struct DiffGraphSolver<'a, T> where T: DiffGraphable {
    diff_graph: &'a DiffGraph<T>,
    diff_graph_paths: BTreeMap<DiffGraphPathParameters, Vec<DiffGraphPoint>>,
}

impl<'a, T> DiffGraphSolver<'a, T> where T: DiffGraphable {
    fn find_min_edit_path(&self) -> Vec<DiffGraphPoint> {
        let MAX = self.diff_graph.target_length() + self.diff_graph.original_length();
        let mut V = vec![0 as usize; 2 * MAX + 1];
        let mut P = vec![Vec::<DiffGraphPoint>::new(); 2 * MAX + 1];

        for d in 0..MAX {
            for k in (MAX - d..MAX + d + 1).step_by(2) {
                let j;
                let i;
                let mut path;
                if k == MAX - d || k != MAX + d && V.get(k - 1) < V.get(k + 1) {
                    j = V.get(k + 1).unwrap().clone();
                    path = P.get(k + 1).unwrap().clone();
                } else {
                    j = V.get(k - 1).unwrap().clone() + 1;
                    path = P.get(k - 1).unwrap().clone();
                }
                i = (j + MAX) - k;
                let p = self.diff_graph.snake_descent(&DiffGraphPoint::from((i, j)));
                path.push(p);
                V.get_mut(k).map(|x| *x = p.j);
                P.get_mut(k).map(|x| *x = path);
                if p.i >= self.diff_graph.target_length()
                    && p.j >= self.diff_graph.original_length() {
                    return P.remove(k);
                }
            }
        }
        panic!("couldn't find an edit path");
    }
}

impl<'a, T> From<&'a DiffGraph<T>> for DiffGraphSolver<'a, T> where T: DiffGraphable {
    fn from(diff_graph: &'a DiffGraph<T>) -> Self {
        Self { diff_graph, diff_graph_paths: Default::default() }
    }
}

impl std::fmt::Display for DiffGraph<u8> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let orig_string = String::from_utf8_lossy(self.orig_lines.as_slice());
        let target_string = String::from_utf8_lossy(self.target_lines.as_slice());

        for orig_char in orig_string.graphemes(false) {
            f.write_str("     ");
            f.write_str(orig_char);
        }

        for (i, target_char) in target_string.graphemes(false).enumerate() {
            f.write_str("\n");
            f.write_str("  ");
            for orig_char in orig_string.graphemes(false) {
                if orig_char == target_char {
                    f.write_str("\\     ");
                } else {
                    f.write_str("      ");
                }
            }
            f.write_str("\n");
            f.write_str(target_char);
            f.write_str("    ");

            for orig_char in orig_string.graphemes(false) {
                f.write_str("X     ");
            }
        }

        Ok(())
    }
}

fn main() {
    let mut orig_string = Vec::from(String::from("hakunamatata"));
    let mut target_string = Vec::from(String::from("hasta la vista"));
    let diff_graph = DiffGraph::new(orig_string, target_string);
    let solver = DiffGraphSolver::from(&diff_graph);
    println!("{}", diff_graph);
    println!("{:#?}", solver.find_min_edit_path());
}
