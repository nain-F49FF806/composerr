# composerr

Why build bespoke errors, when you can compose!

## About

Many rust libraries have a single large error enum with error variants being all the different failure modes *of the library*.
Sometimes this is not specific enough.

For example consider the error enum of the wonderful sqlx library, [`sqlx::Error`](https://docs.rs/sqlx/latest/sqlx/error/enum.Error.html).  
`sqlx::Error` has a total of 16 variants.  Including error variants such as `Configuration`, `Io`, `Tls`, `RowNotFound`, `ColumnNotFound`, `ColumnDecode` etc.

Not every library function can fail in all the different modes in this enum. But may error to a subset of them.  
For example, when executing a query

```rust
    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;
```

Given that the result type of `fetch_one` is `Result<O, sqlx::Error>`, say we want to handle the error,

- What might be the returned error variant?
- Are all of 16 enum variants equally likely and need to be handled?

Thankfully, documentation for this function, has a little helpful note

> Execute the query, returning the first row or Error::RowNotFound otherwise.

That's good. But this kind of information could be at the type level itself.
Also, `Error::RowNotFound` is a variant that probably does not happen in other scenarios,
like when setting up the database connection. So we could have had a smaller error enum there,
without this specific `Error::RowNotFound` variant in the mix.

You might be thinking. Oh so are you suggesting we write custom, more specific error enums for each function?
That seems like a chore. Too much...

How about.. if it's very easy?

## Usage

```rust
use composerr::compose_errors;
use rand::Rng;
use std::{fmt::Error as FmtError, io::Error as IoError};
use thiserror::Error;

#[compose_errors]
#[errorset(IoError, FmtError)]  // <-- This easy!
fn moody_task_do() -> Result<(), _> {
    let mut rng = rand::thread_rng();
    // Randomly decide if to error
    if rng.gen::<bool>() {
        let mood = if rng.gen::<bool>() {
            // not feeling like expressing
            FmtError.into()
        } else {
            // stuck on past mood
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

### Black magic?

Basically, we are doing error composition.
You can define your individual base errors anyway you like.
The only requirement is that they implement the `std::error::Error` trait.

Then for each function you want to provide precise information. Just declare the `errorset`.  
And leave the return Error type as blank (aka underscore) `_`.

The macro will construct the necessary error enum for you!  
Under the hood it uses [`thiserror`](https://docs.rs/thiserror/latest/thiserror/).

> [!TIP]  
> You don't have to abandon your superb all-in-one error set in one go or make huge refactors.  
> You can gradually add error precision to some functions where it make sense using composerr.

### Where is the macro supported

The macro works for

- Simple, named *bare* functions
- Functions in *`impl`* blocks
- Functions in *trait* definitions

#### Trait example

```rust
#[compose_errors] // <- Macro invoke on trait
trait MyTrait {
    #[errorset{IOError, BugsBunnyError}] // <-- Declare errorset on individual functions
    fn function1(&self) -> Result<(), _> ;

    // You can have functions not using errorset helper. Mix and match is okay. 
    fn function2(&self);

    #[errorset[IOError, ZFhOt01Rdb0Error]]
    fn function3(&self) -> Result<(), _> ;
}
```

<!--// !Note: Todo [ Mark base errors with base trait ?]
// Then when we do flat you can flatten till base.

// or..
// Mark compositions.
// so when doing error_compose_flat , you can break composition to constituents, and merge those.
-->