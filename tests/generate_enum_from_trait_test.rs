use composerr::generate_enum_from_trait;

#[generate_enum_from_trait]
trait MyTrait {
    fn function1(&self);
    fn function2(&self);
}

fn main() {
    // This is just a placeholder to compile the program.
    // The real test is in the generated enum.
}
