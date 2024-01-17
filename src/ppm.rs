use std::{fmt::{Display, Write}, path::Path};

pub struct Ppm {
    width: usize,
    height: usize,
    pix: Vec<u8>
}

impl Ppm {
    pub fn new(width: usize, height: usize, pix: Vec<u8>) -> Self {
        Ppm{width, height, pix}
    }

    pub fn write_impl(&self, path_str: &str) {
        let mut buf = String::new();
        write!(buf, "{}", self).unwrap();
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() { 
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, buf).unwrap();
    }

    pub fn write(width: usize, height: usize, pix: Vec<u8>, path_str: &str) {
        Ppm::new(width, height, pix).write_impl(path_str);
    }
}

impl Display for Ppm {
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
    use std::fmt::Write;

    let ppm = Ppm{width: 1, height: 1, pix: vec![0, 127, 255]};

    let mut buf = String::new();
    write!(&mut buf, "{}", ppm).unwrap();
    assert_eq!(buf, "P3\n1 1\n255\n0 127 255\n");
}