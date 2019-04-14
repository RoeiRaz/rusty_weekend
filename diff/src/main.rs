/*
 * BSD 2-Clause License
 *
 * Copyright (c) @year, Roei Rosenzweig
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 *  Redistributions of source code must retain the above copyright notice, this
 *   list of conditions and the following disclaimer.
 *
 *  Redistributions in binary form must reproduce the above copyright notice,
 *   this list of conditions and the following disclaimer in the documentation
 *   and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */


extern crate unicode_segmentation;

use core::fmt::Write;
use std::cmp::min;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque, BTreeSet};
use std::fmt::{Debug, Error, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Display;
use std::vec::Vec;

use unicode_segmentation::UnicodeSegmentation;
use std::iter::Map;

trait DiffGraphable: Debug + Eq + Default + Copy + Clone {}
impl<T: Debug + Eq + Default + Copy + Clone> DiffGraphable for T {}

// ########################## [DiffGraph] ##########################################################

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

// ########################## [DiffGraphPoint / DiffGraphPathParameters] ###########################

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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

// ########################## [DiffGraphSolver] ####################################################

type EditPath = Vec<DiffGraphPoint>;

struct DiffGraphSolver<'a, T> where T: DiffGraphable {
    diff_graph: &'a DiffGraph<T>,
    diff_graph_paths: BTreeMap<DiffGraphPathParameters, Vec<DiffGraphPoint>>,
}

impl<'a, T> DiffGraphSolver<'a, T> where T: DiffGraphable {
    fn find_min_edit_path(&self) -> EditPath {
        let MAX = self.diff_graph.target_length() + self.diff_graph.original_length() + 2;
        let mut V = vec![0 as usize; 2 * MAX + 1];
        let mut P = vec![Vec::<DiffGraphPoint>::new(); 2 * MAX + 1];

        for d in 0..MAX {
            for k in (MAX - d..MAX + d + 1).step_by(2) {
                let j;
                let i;
                let mut path;
                // determines the j (x) value of the next point by trying to extend the best
                // d-1 path from the previous iteration.
                if k == MAX - d || k != MAX + d && V.get(k - 1) < V.get(k + 1) {
                    j = V.get(k + 1).unwrap().clone();
                    path = P.get(k + 1).unwrap().clone();
                } else {
                    j = V.get(k - 1).unwrap().clone() + 1;
                    path = P.get(k - 1).unwrap().clone();
                }
                // infer the i (y) value
                i = (j + MAX) - k;

                // push next point into path
                let next_point = DiffGraphPoint::from((i, j));
                path.push(next_point.clone());

                // make a snake descent if possible, add last descent point to the path
                let descent_point = self.diff_graph.snake_descent(&DiffGraphPoint::from((i, j)));
                if descent_point != next_point {
                    path.push(descent_point.clone())
                }

                // there must be a prettier way of doing this
                P.get_mut(k).map(|x| *x = path);
                V.get_mut(k).map(|x| *x = descent_point.j);

                // if we reached the target \ overshoot in both axes, terminate
                if descent_point.i >= self.diff_graph.target_length()
                    && descent_point.j >= self.diff_graph.original_length() {
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

// ########################## [DiffScript] #########################################################

#[derive(Debug, Clone)]
enum DiffCommand<T> where T: DiffGraphable {
    Delete,
    Insert(T),
}

#[derive(Debug, Clone)]
struct DiffScript<T> where T: DiffGraphable {
    commands: Vec<(usize, DiffCommand<T>)>,
}

impl<T> DiffScript<T> where T: DiffGraphable {
    fn new(target: &[T], edit_path: &EditPath) -> Self {
        let mut res = DiffScript {commands: Default::default()};
        for (prev, curr) in edit_path.iter().zip(edit_path.iter().skip(1)) {
            let mut indexed_command;

            // insertion
            if curr.i == prev.i + 1 && curr.j == prev.j {
                let orig_element = target.get(prev.i).unwrap().clone();
                indexed_command = (prev.j, DiffCommand::Insert(orig_element));
                res.commands.push(indexed_command);
            }
            // deletion
            else if curr.i == prev.i && curr.j == prev.j + 1 {
                indexed_command = (prev.j, DiffCommand::Delete);
                res.commands.push(indexed_command);
            }
        }
        return res;
    }

    fn apply_copy(&self, target: &[T]) -> Vec<T> {
        let mut copied_target: Vec<T> = target.to_vec();
        let mut aggregated_offset: isize = 0;
        for (i, s) in self.commands.iter() {
            let fixed_index: usize = (i.clone() as isize + aggregated_offset) as usize;
            match s {
                DiffCommand::Insert(e) => {
                    copied_target.insert(fixed_index, e.clone());
                    aggregated_offset = aggregated_offset + 1;
                },
                DiffCommand::Delete => {
                    copied_target.remove(fixed_index);
                    aggregated_offset = aggregated_offset - 1;
                }
            }
        }
        return copied_target;
    }
}

// ########################## [Support] ############################################################

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

// ########################## [Remove this] ########################################################

fn main() {
    let mut orig_string = Vec::from(String::from("t1e2s3t4"));
    let mut target_string = Vec::from(String::from("t1e3s2ttt"));
    let diff_graph = DiffGraph::new(orig_string.clone(), target_string.clone());
    let solver = DiffGraphSolver::from(&diff_graph);
    let diffscript = DiffScript::new(&diff_graph.target_lines, &solver.find_min_edit_path());
    let result = String::from_utf8(diffscript.apply_copy(&orig_string));
    println!("{:#?}", result);
}
