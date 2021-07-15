#[ctor::ctor]
fn init_tests() {
    color_backtrace::install();
}
pub(crate) mod segment;
pub(crate) mod sources;
