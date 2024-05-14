use std::fmt::Display;

use composerr::generate_enum;

#[generate_enum]
trait MyTrait {
    #[select]
    fn function1(&self);
    fn function2(&self);
    #[select]
    fn function3(&self);
}

struct Dummy;

#[generate_enum]
impl Dummy {
    fn function4(&self) {}
    #[select]
    fn function5(&self) {}
}

impl Display for Dummy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dummy")
    }
}

#[generate_enum]
#[select]
fn main() {
    // This is just a placeholder to compile the program.
    // The real test is in the generated enum.
}
