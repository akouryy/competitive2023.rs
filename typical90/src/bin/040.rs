#![allow(
    dead_code,
    non_snake_case,
    unused_imports,
    unused_macros,
    clippy::uninlined_format_args,
    clippy::upper_case_acronyms
)]

use itertools::{iproduct, izip, Itertools};
use petgraph::{prelude::*, stable_graph::IndexType, visit::IntoNodeReferences};
use proconio::{input, marker::Usize1};
use rand::Rng;
use std::cmp::{max, min};

const IINF: isize = 1 << 60;
const UINF: usize = 1 << 60;
const EPS: f64 = 1e-20;
const MOD: usize = 1e9 as usize + 7;

macro_rules! d {
    ($x0:expr $(, $xs:expr)* $(,)?) => {
        #[cfg(debug_assertions)]
        eprintln!(concat!(stringify!($x0), "={:?}", $(",  ", stringify!($xs), "={:?}"), *), &$x0, $(&$xs),*);
    };
}

fn dinic(G: &DiGraph<(), usize, usize>, start: usize, goal: usize) -> (usize, Vec<((usize, usize), usize)>) {
    let nv = G.node_count();
    let mut edge_list = vec![vec![]; nv];
    let mut maxflow = 0;
    if start < nv && goal < nv {
        for e in G.raw_edges().iter() {
            let (i, j) = (e.source().index(), e.target().index());
            let (eij, eji) = (edge_list[i].len(), edge_list[j].len());
            edge_list[i].push((j, e.weight, eji, Outgoing));
            edge_list[j].push((i, 0, eij, Incoming));
        }
        loop {
            let mut dist = vec![-1; nv];
            dist[start] = 0;
            let mut bfs = std::collections::VecDeque::from(vec![start]);
            'bfs: while let Some(i) = bfs.pop_front() {
                for &(j, c, _, _) in edge_list[i].iter() {
                    if c > 0 && dist[j] < 0 {
                        dist[j] = dist[i] + 1;
                        if j == goal {
                            break 'bfs;
                        }
                        bfs.push_back(j);
                    }
                }
            }
            if dist[goal] < 0 {
                break;
            }
            let mut path = Vec::<(usize, usize)>::new();
            let mut dfs = vec![Some((start, None))];
            while let Some(next) = dfs.pop() {
                if let Some((i, from)) = next {
                    path.extend(from);
                    if i == goal {
                        let (f, prune_under) = path
                            .iter()
                            .map(|&(j, ejk)| (edge_list[j][ejk].1, edge_list[j][ejk].0))
                            .min()
                            .unwrap();
                        maxflow += f;
                        path.iter().for_each(|&(j, ejk)| {
                            let (k, _, ekj, _) = edge_list[j][ejk];
                            edge_list[j][ejk].1 -= f;
                            edge_list[k][ekj].1 += f;
                        });
                        if prune_under != i {
                            while {
                                match dfs.pop().unwrap() {
                                    Some((_, _)) => true,
                                    None => path.pop().unwrap().0 != prune_under,
                                }
                            } {}
                        }
                    }
                    dfs.push(None);
                    for (eij, &(j, c, _, _)) in edge_list[i].iter().enumerate() {
                        if c > 0 && dist[i] < dist[j] {
                            dfs.push(Some((j, Some((i, eij)))));
                        }
                    }
                } else {
                    path.pop();
                }
            }
        }
    }
    let used_edges = edge_list
        .iter()
        .enumerate()
        .flat_map(|(j, es)| {
            es.iter()
                .flat_map(move |&(i, c, _, dir)| if dir == Incoming && c > 0 { Some(((i, j), c)) } else { None })
        })
        .collect();
    (maxflow, used_edges)
}

#[deny(dead_code)]
#[proconio::fastout]
fn main() {
    input! {
        N: usize,
        W: usize,
        A: [usize; N],
        C: [[Usize1]; N],
    }

    let start = N;
    let goal = N + 1;

    let G = DiGraph::from_edges(
        (0..N)
            .map(|i| (start, i, A[i]))
            .chain((0..N).map(|i| (i, goal, W)))
            .chain((0..N).flat_map(|i| C[i].iter().map(move |&j| (j, i, UINF)))),
    );

    let (flow, _) = dinic(&G, start, goal);

    println!("{}", A.iter().sum::<usize>() - flow);
}
