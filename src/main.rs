use std::io;
use quadedge_rs::Manifold;

fn main() {
    let m: Manifold<(), ()> = Manifold::default();
    let a = m.make_quad();
    let b = m.make_quad();
    let c = m.make_quad();
    let d = m.make_quad();
    let e = m.make_quad();
    let f = m.make_quad();
    let g = m.make_quad();
    let h = m.make_quad();
    a.orig().splice(b.orig());
    b.dest().splice(c.orig());
    c.dest().splice(d.orig());
    d.dest().splice(e.orig());
    e.orig().splice(e.dest());
    e.dest().splice(f.orig());
    f.dest().splice(e.orig());
    g.dest().splice(a.orig());
    g.dest().splice(h.orig());
    h.dest().splice(f.orig());

    let mut buf = io::BufWriter::new(io::stdout());
    m.export(&mut buf).unwrap();
}
