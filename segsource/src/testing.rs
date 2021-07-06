#[ctor::ctor]
fn init_tests() {
    color_backtrace::install();
}
mod segment;
