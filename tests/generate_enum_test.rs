use composerr::generate_enum;
use std::fmt::{Display, Error};
use std::io::Error as IOError;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Based Error this")]
pub struct BasedError;

#[generate_enum]
trait MyTrait {
    #[errorset{IOError, BasedError}]
    fn function1(&self);
    fn function2(&self);
    #[errorset[IOError]]
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

#[test]
#[generate_enum]
#[errorset(IOError, BasedError)]
fn main() {
    // This is just a placeholder to compile the program.
    // The real test is in the generated enum.

    let _err: MainError = BasedError.into();

    let err = MainError::BasedError(BasedError);
    let _based: BasedError = err.try_into().unwrap();

    let err = MainError::BasedError(BasedError);
    let _ioerr: IOError = err.try_into().expect("This will fail");
}
