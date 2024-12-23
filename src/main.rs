use motion_sketch::run;
use pollster::block_on;
fn main() {
    block_on(run());
}