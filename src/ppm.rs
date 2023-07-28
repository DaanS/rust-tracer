use std::fmt::Display;
use std::fmt::Write;

pub struct Ppm<'a> {
    width: usize,
    height: usize,
    pix: &'a Vec<u8>
}

impl<'a> Ppm<'a> {
    pub fn new(width: usize, height: usize, pix: &'a Vec<u8>) -> Self {
        Ppm{width, height, pix}
    }
}

impl Display for Ppm<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert_eq!(self.pix.len(), self.width * self.height * 3);
        write!(f, "P3\n{} {}\n255\n", self.width, self.height)?;
        for col in self.pix.chunks(3) {
            match col {
                &[r, g, b] => write!(f, "{r} {g} {b}\n")?,
                _ => panic!("expected &[r, g, b], found: {:?}", col)
            }
        }
        Ok(())
    }
}

#[test]
fn test_to_string() {
    let ppm = Ppm{width: 1, height: 1, pix: &vec![0, 127, 255]};

    let mut buf = String::new();
    write!(&mut buf, "{}", ppm).unwrap();
    assert_eq!(buf, "P3\n1 1\n255\n0 127 255\n");
}