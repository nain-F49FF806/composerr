use std::fmt::Display;
use std::io::Error;

use composerr::generate_enum;

#[generate_enum]
trait MyTrait {
    #[errorset{Error, Display}]
    fn function1(&self);
    fn function2(&self);
    #[errorset[]]
    fn function3(&self);
}

struct Dummy;

#[generate_enum]
impl Dummy {
    fn function4(&self) {}
    #[errorset{}]
    fn function5(&self) {}
}

impl Display for Dummy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dummy")
    }
}

mod foo {
    pub struct Dummy2;
}

#[generate_enum]
impl foo::Dummy2 {
    fn function6(&self) {}
    #[errorset()]
    fn function7(&self) {}
}

#[generate_enum]
#[errorset()]
fn main() {
    // This is just a placeholder to compile the program.
    // The real test is in the generated enum.
}
