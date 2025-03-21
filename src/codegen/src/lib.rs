#[macro_use]
mod util;

mod model;

mod names;
use names::Names;

mod out;
pub use out::{ModelOutput, Output};

use toasty_core::schema::*;

use std::rc::Rc;

/// Generate client code for a schema
pub fn generate(schema: &app::Schema, in_macro: bool) -> Output<'_> {
    // Compute names of structs, mods, etc...
    let names = Rc::new(Names::from_schema(schema));

    let models = schema
        .models()
        .map(|model| model::generate(model, names.clone(), in_macro))
        .collect();

    Output { models }
}
