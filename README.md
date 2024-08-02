# composerr

Why write bespoke error enums, when you can compose!

## About

Many rust libraries have a single large error enum. With error variants being *all* the different failure modes *of the library*.  
Sometimes this is can dilute locally relevant information.

For example consider the error enum of the wonderful sqlx library, [`sqlx::Error`](https://docs.rs/sqlx/latest/sqlx/error/enum.Error.html).

```rust

#[non_exhaustive]
pub enum Error {
    Configuration(Box<dyn Error + Send + Sync>),
    Database(Box<dyn DatabaseError>),
    Io(Error),
    Tls(Box<dyn Error + Send + Sync>),
    Protocol(String),
    RowNotFound,
    TypeNotFound {
        type_name: String,
    },
    ColumnIndexOutOfBounds {
        index: usize,
        len: usize,
    },
    ColumnNotFound(String),
    ColumnDecode {
        index: String,
        source: Box<dyn Error + Send + Sync>,
    },
    Encode(Box<dyn Error + Send + Sync>),
    Decode(Box<dyn Error + Send + Sync>),
    AnyDriverError(Box<dyn Error + Send + Sync>),
    PoolTimedOut,
    PoolClosed,
    WorkerCrashed,
    Migrate(Box<MigrateError>),
}

```

`sqlx::Error` has a total of 16 variants.  Including disparate error variants such as `Configuration`, `Tls`, `RowNotFound`, `ColumnDecode` etc.

Not every function emits errors in all the different variant modes of this enum. But may error to only a subset of them.  
For example, when executing a query

```rust
// Make a simple query to return the given parameter
let row: (i64,) = sqlx::query_as("SELECT $1")
    .bind(150_i64)
    .fetch_one(&pool).await?;
```

Given that the result type of `fetch_one` is `Result<O, sqlx::Error>`, say we want to handle the error,

- What might be the returned error variant?
- Are all of 16 enum variants equally likely and need to be handled?

Thankfully, documentation for this function, has a little helpful note

> Execute the query, returning the first row or Error::RowNotFound otherwise.

That's good. But this kind of information could be available at the type level itself.
Also, `Error::RowNotFound` is a variant that probably does not happen in most other scenarios.
Like when setting up the database connection. So we are carrying it around unnecessarily and could do with a smaller error enum elsewhere.

You might be thinking. Oh so are you suggesting we write custom, more specific error enums for each function?
That seems like a chore. Too much...

How about.. if it's very easy?

## Demo

```rust
use composerr::compose_errors;
use rand::Rng;
use std::{fmt::Error as FmtError, io::Error as IoError};

#[compose_errors]
#[errorset(IoError, FmtError)]  // <-- This easy!
fn moody_task_do() -> Result<(), _> {
    let mut rng = rand::thread_rng();
    // Randomly decide if to error
    if rng.gen::<bool>() {
        let mood = if rng.gen::<bool>() {
            // not feeling like expressing today
            FmtError.into()
        } else {
            // stuck on a past mood
            IoError::last_os_error().into()
        };
        return Err(mood);
    }
    // Do something cool
    Ok(())
}

fn main() {
    let res: Result<(), MoodyTaskDoError> = moody_task_do();
    match res.unwrap_err() {
        MoodyTaskDoError::IoError(e) => println!("an io error {}", e),
        MoodyTaskDoError::FmtError(e) => println!("a formatting error {}", e),
    }
}
```

### What black magic is this?

Basically, we are doing error compositions.
You can define your individual base errors anyway you like.
The only requirement is that they implement the `std::error::Error` trait.

Then for each function that you want to provide precise error information for. Just declare the `errorset`.  
Leave the return Error type as inferred ( `_` ) so the macro can replace it with the composed error enum.

The macro will construct the necessary error enum for you!  
Under the hood it uses [`thiserror`] for the error composition, so your public api remains similarly unpolluted.

[`thiserror`]: https://docs.rs/thiserror/latest/thiserror/

> [!TIP]  
> You don't have to abandon your superb all-in-one error set in one go or make huge refactors.  
> You can gradually add error precision to some functions where it make sense using composerr.

### Where is the macro supported?

The macro works for

- Simple, named *bare* functions
- Functions in *`impl`* blocks
- Functions in *trait* definitions

#### Trait example

```rust
#[compose_errors] // <- Macro invoke on trait
trait MyTrait {
    #[errorset{IOError, BugsBunnyError}] // <-- Declare errorset for individual functions
    fn function1(&self) -> Result<(), _> ;

    // You can have functions not using errorset helper. Mix and match is okay. 
    fn function2(&self) -> Result<(), String>;

    #[errorset[IOError, ZFhOt01Rdb0Error]]
    fn function3(&self) -> Result<(), _> ;
}
```

### Defining composable / base errors

Only requirement for an error to be composable is that it implements `std::error::Error` trait.
One can use the popular [`thiserror`] library to create base errors, or implement the trait manually.

#### Impl example with custom errors defined using `thiserror`

```rust
mod my_base_errors {
    /// Collection of base error variants used in my library

    #[derive(thiserror::Error, Debug)]
    #[error("Based Error this")]
    pub struct BasedError;

    #[derive(thiserror::Error, Debug)]
    #[error(transparent)]
    pub struct IoError(#[from] std::io::Error);

    #[derive(thiserror::Error, Debug)]
    pub enum ConfigError {
        #[error("Please provide a config file")]
        NotFound(#[from] std::io::Error),
        #[error("Required fields are missing: `{0:?}`")]
        MissingFields(Vec<String>),
    }
}
use my_base_errors::*;


pub struct Foo;

#[compose_errors]
impl Foo {
    fn function4() -> Result<(), IoError> {
        Ok(())
    }

    #[errorset(ConfigError, BasedError)]
    fn function5(&self) -> Result<String, _> {
        Ok("Am ok".to_owned())
    }
}

```

<!--// !Note: Todo [ Mark base errors with base trait ?]
// Then when we do flat you can flatten till base.

// or..
// Mark compositions.
// so when doing error_compose_flat , you can break composition to constituents, and merge those.
-->

## Install

Composerr is in very early stage of development, so we recommend you install from source repo.

From command line

```bash
cargo add --git https://github.com/nain-F49FF806/composerr.git
```

Cargo.toml

```toml
[dependencies]
composerr = { git = "https://github.com/nain-F49FF806/composerr.git" }
```
