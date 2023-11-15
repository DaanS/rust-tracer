
pub struct MyStruct {}

impl MyStruct {
    pub fn new() -> MyStruct {
        println!("new called");
        MyStruct{}
    }
}

impl Clone for MyStruct {
    fn clone(&self) -> Self {
        println!("clone called");
        Self{}
    }
}

pub fn structer() -> &'static MyStruct {
    todo!();
    thread_local! {
        static MS: MyStruct = MyStruct::new();
    }
}

fn main() {
    let ms = structer();
}