#![allow(
    dead_code,
    non_snake_case,
    unused_imports,
    unused_macros,
    clippy::uninlined_format_args,
    clippy::upper_case_acronyms
)]

use itertools::Itertools;
use num::Num;
use petgraph::prelude::*;
use proconio::{fastout, input, marker::Usize1};
use rand::Rng;
use std::{
    cmp::{max, min},
    fmt,
    ops::Range,
};

const IINF: isize = 1 << 60;
const UINF: usize = 1 << 60;
const EPS: f64 = 1e-20;
const MOD: usize = 1e9 as usize + 7;

trait Monoid {
    type Val: Clone + Eq;
    fn op(a: &Self::Val, b: &Self::Val) -> Self::Val;
    fn id() -> Self::Val;
}
struct MaxMonoid {}
impl Monoid for MaxMonoid {
    type Val = usize;
    fn op(a: &usize, b: &usize) -> usize {
        *a.max(b)
    }
    fn id() -> usize {
        0
    }
}
struct RightMonoid {}
impl Monoid for RightMonoid {
    type Val = usize;
    fn op(_: &usize, b: &usize) -> usize {
        *b
    }
    fn id() -> usize {
        0
    }
}
/// 遅延セグメント木
/// - `(M, ⊕)`, `(Lazy, ∘)`はモノイド (可換とは限らない)。
/// - 準同型性α: `apply(apply(m,l1),l2) = apply(m,l1∘l2)`
/// - 準同型性β: `apply(m1⊕m2,l) = apply(m1,l)⊕apply(m2,l)`
struct SegmentTree<M, Lazy>
where
    M: Monoid,
    Lazy: Monoid,
{
    n: usize,
    height: usize,
    apply: fn(&M::Val, &Lazy::Val) -> M::Val,
    data: Vec<M::Val>,    // 1-based
    lazy: Vec<Lazy::Val>, // 1-based
}
impl<M, Lazy> SegmentTree<M, Lazy>
where
    M: Monoid,
    Lazy: Monoid,
{
    fn new(n: usize, apply: fn(&M::Val, &Lazy::Val) -> M::Val) -> Self {
        let n = n.next_power_of_two();
        let height = (0..62).find(|&x| n == 1 << x).unwrap() + 1;
        Self { n, height, apply, data: vec![M::id(); 2 * n], lazy: vec![Lazy::id(); 2 * n] }
    }
    fn new_with_data(data: Vec<M::Val>, apply: fn(&M::Val, &Lazy::Val) -> M::Val) -> Self {
        let mut st = Self::new(data.len(), apply);
        for (i, d) in data.iter().enumerate() {
            st.data[i + st.n] = d.clone();
        }
        for i in (1..st.n).rev() {
            st.data[i] = M::op(&st.data[i << 1], &st.data[i << 1 | 1]);
        }
        st
    }
    #[inline(always)]
    fn propagate(&mut self, k: usize) {
        if self.lazy[k] == Lazy::id() {
            return;
        }
        if k < self.n {
            self.lazy[k << 1] = Lazy::op(&self.lazy[k << 1], &self.lazy[k]);
            self.lazy[k << 1 | 1] = Lazy::op(&self.lazy[k << 1 | 1], &self.lazy[k]);
        }
        self.data[k] = (self.apply)(&self.data[k], &self.lazy[k]);
        self.lazy[k] = Lazy::id();
    }
    #[inline(always)]
    fn propagate_all(&mut self, range: &Range<usize>) {
        let l = range.start + self.n;
        let r = range.end + self.n - 1;
        for d in (0..self.height).rev() {
            self.propagate(l >> d);
            self.propagate(l >> d ^ 1); // XXX
            self.propagate(r >> d);
            self.propagate(r >> d ^ 1); // XXX
        }
    }
    fn query(&mut self, range: Range<usize>) -> M::Val {
        self.propagate_all(&range);
        let mut l = range.start + self.n;
        let mut r = range.end + self.n - 1;
        let mut ans_l = M::id();
        let mut ans_r = M::id();
        while l <= r {
            if l & 1 == 1 {
                self.propagate(l);
                ans_l = M::op(&ans_l, &self.data[l]);
                l += 1;
            }
            if r & 1 == 0 {
                self.propagate(r);
                ans_r = M::op(&self.data[r], &ans_r);
                r -= 1;
            }
            l >>= 1;
            r >>= 1;
        }
        M::op(&ans_l, &ans_r)
    }
    fn update(&mut self, range: Range<usize>, val: Lazy::Val) {
        self.propagate_all(&range);
        let mut l = range.start + self.n;
        let mut r = range.end + self.n - 1;
        while l <= r {
            if l & 1 == 1 {
                self.lazy[l] = Lazy::op(&self.lazy[l], &val);
                self.propagate(l);
                l += 1;
            }
            if r & 1 == 0 {
                self.lazy[r] = Lazy::op(&self.lazy[r], &val);
                self.propagate(r);
                r -= 1;
            }
            l >>= 1;
            r >>= 1;
        }
        let l = range.start + self.n;
        let r = range.end + self.n - 1;
        for d in 1..self.height {
            if l & ((1 << d) - 1) != 0 {
                let ll = l >> d;
                self.data[ll] = M::op(&self.data[ll << 1], &self.data[ll << 1 | 1]);
            }
            if (r + 1) & ((1 << d) - 1) != 0 {
                let rr = r >> d;
                self.data[rr] = M::op(&self.data[rr << 1], &self.data[rr << 1 | 1]);
            }
        }
    }
}
impl<M, Lazy> fmt::Debug for SegmentTree<M, Lazy>
where
    M: Monoid,
    M::Val: fmt::Debug,
    Lazy: Monoid,
    Lazy::Val: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strs = self
            .data
            .iter()
            .zip(self.lazy.iter())
            .skip(1)
            .map(|(d, l)| format!("{:?}/{:?},", d, l))
            .collect_vec();
        let len = strs.iter().map(|s| s.len()).max().unwrap() + 1;
        for d in 0..self.height {
            for c in strs.iter().take((1 << (d + 1)) - 1).skip((1 << d) - 1) {
                write!(f, "{:width$}", c, width = len << (self.height - 1 - d))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[deny(dead_code)]
#[fastout]
fn main() {
    input! {
        W: usize,
        N: usize,
        LR: [(Usize1, usize); N],
    }

    let mut st = SegmentTree::<MaxMonoid, RightMonoid>::new(W, |&_, &l| l);
    for (l, r) in LR {
        let h = st.query(l..r) + 1;
        st.update(l..r, h);
        // eprintln!("{:?}", st);
        println!("{}", h);
    }
}
