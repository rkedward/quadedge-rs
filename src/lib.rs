#![allow(dead_code)]
use std::cell::Cell;
use std::fmt::{self, Debug, Formatter};
use std::ops::Index;
use typed_arena::Arena;

#[derive(Default)]
pub struct Edge<'m, D> {
    refs: Cell<Option<[&'m Edge<'m, D>; 2]>>,
    data: Cell<Option<&'m D>>,
}

impl<'m, D> Edge<'m, D> {
    #[inline]
    fn next(&self) -> &'m Edge<'m, D> {
        self.refs.get().unwrap()[0]
    }

    #[inline]
    fn rot(&self) -> &'m Edge<'m, D> {
        self.refs.get().unwrap()[1]
    }

    #[inline]
    pub fn set(&self, val: &'m D) {
        self.data.set(Some(val));
    }

    #[inline]
    pub fn splice(&self, other: &'m Edge<'m, D>) {
        let _alpha = self.next().rot();
        let _beta = other.next().rot();
        // todo - impliment splice operator
    }


}

impl<'m, D> PartialEq for Edge<'m, D> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl<'m, D> Debug for Edge<'m, D> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Edge({:#08x?})", self as *const _)
    }
}

#[derive(Default, Debug)]
pub struct QuadEdge<'m, D> {
    edges: [Edge<'m, D>; 4],
}

impl<'m, D> QuadEdge<'m, D> {
    fn init(&'m self, edges: [&'m Edge<'m, D>; 4]) {
        for ind in 0..=3 {
            self.edges[ind].refs.set(Some([edges[ind], &self[ind + 1]]));
        }
    }

    fn set(&self, vals: [&'m D; 4]) {
        for ind in 0..=3 {
            self.edges[ind].set(vals[ind]);
        }
    }

    #[inline]
    fn onext(&self) -> &'m Edge<'m, D> {
        self.edges[0].next()
    }
}

impl<'m, D> PartialEq for QuadEdge<'m, D> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl<'m, D> Index<usize> for QuadEdge<'m, D> {
    type Output = Edge<'m, D>;

    fn index(&self, ind: usize) -> &Self::Output {
        &self.edges[(ind).rem_euclid(4)]
    }
}

pub struct Manifold<'m, D> {
    quads: Arena<QuadEdge<'m, D>>,
    data: Arena<D>,
}

impl<'m, D: Default> Manifold<'m, D> {
    pub fn new() -> Self {
        Self {
            quads: Arena::new(),
            data: Arena::new(),
        }
    }

    pub fn make_quad(&'m self) -> &'m QuadEdge<'m, D> {
        let q = self.quads.alloc(QuadEdge::default());
        q.init([&q[0], &q[3], &q[2], &q[1]]);
        q
    }

    pub fn make_datum(&self, val: D) -> &mut D {
        self.data.alloc(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Datum;

    #[test]
    fn check_internal_operations() {
        let m: Manifold<()> = Manifold::new();
        let q = m.make_quad();
        assert_eq!(q[0].next(), &q[0]);
        assert_eq!(q[1].next(), &q[3]);
        assert_eq!(q[2].next(), &q[2]);
        assert_eq!(q[3].next(), &q[1]);
        assert_eq!(q[4].next(), &q[0]);
        assert_eq!(q[0].rot(), &q[1]);
        assert_eq!(q[1].rot(), &q[2]);
        assert_eq!(q[2].rot(), &q[3]);
        assert_eq!(q[3].rot(), &q[0]);
        assert_eq!(q[0].rot().next(), &q[3]);
        assert_eq!(q[1].next().rot(), &q[0]);
        assert_eq!(q.onext(), q[0].next());
    }

    #[test]
    fn check_splice_operations() { 
        let m: Manifold<()> = Manifold::new();
        let q0 = m.make_quad();
        let q1 = m.make_quad();
        q0[0].splice(&q1[1]);
        q0[0].splice(&q1[1]);

        // TodDo: Add assert statememts
    }


}
