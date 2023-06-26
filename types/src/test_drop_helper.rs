// NOTE: Useing drop trait for cleaning up env
// https://doc.rust-lang.org/std/ops/trait.Drop.html

use std::path::PathBuf;

struct DropTemp(PathBuf);
impl Drop for DropTemp {
    fn drop(&mut self) {
      println!("we dropped, running cleanup");
      self.maybe_cleanup();
    }
}

impl DropTemp {
  pub fn new_in_crate(dir_string: &str) -> Self {
    let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(dir_string);
    DropTemp(test_path)
  }
  
  pub fn maybe_cleanup(&self) {
    if self.0.exists() {
      println!("\n RUNNING CLEANUP \n");
      std::fs::remove_dir_all(&self.0).unwrap();
    }
  }
}
