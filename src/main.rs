use quadedge_rs::Manifold;

fn main() {
    let manifold: Manifold<()> = Manifold::new();
    let q = manifold.make_quad();
    println!("{:?}", q);
}
