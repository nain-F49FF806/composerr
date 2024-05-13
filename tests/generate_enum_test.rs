use composerr::generate_enum;

#[generate_enum]
trait MyTrait {
    fn function1(&self);
    fn function2(&self);
}

struct Dummy;

#[generate_enum]
impl Dummy {
    fn function3(&self) {}
    fn function4(&self) {}
}

fn main() {
    // This is just a placeholder to compile the program.
    // The real test is in the generated enum.
}
