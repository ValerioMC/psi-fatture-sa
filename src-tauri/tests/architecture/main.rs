//! Rules the compiler cannot see inside one crate: which layer may name which, and which
//! domain may use which. They read the `crate::app::` paths of every file under `src/app`,
//! so a dependency pointing the wrong way fails here instead of in review.

mod app_reference;
mod domain_edges;
mod layer_direction;
mod source_file;
