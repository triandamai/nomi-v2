// The translations are compiled in (rust_i18n::i18n!), so editing one must rebuild this crate.
fn main() {
    println!("cargo:rerun-if-changed=locales");
}
