use std::path::Path;

use crate::git::init_submodules;

pub(crate) fn invoke(root_path: &Path) {
    println!("Running setup command");
    init_submodules(root_path);
}
