pub trait Ploop<const Z: usize> {
    fn ploop(&self, x: usize, y: usize) -> String;
}

struct Plooper {}

impl<const Z: usize> Ploop<Z> for Plooper {
    fn ploop(&self, x: usize, y: usize) -> String {
        format!("Ploop at {}, {}, {}", x, y, Z)
    }
}

fn main() {
    let pl = Plooper {};
    println!("{}", Ploop::<5>::ploop(&pl, 3, 4));
}