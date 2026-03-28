pub mod node;

use std::path::Path;

pub fn paths_equal(a: &Path, b: &Path) -> bool {
    a == b
        || dunce::canonicalize(a)
            .ok()
            .zip(dunce::canonicalize(b).ok())
            .is_some_and(|(a, b)| a == b)
}
