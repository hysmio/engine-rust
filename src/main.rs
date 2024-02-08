use engine_rust::run;
use pollster;

fn main() {
    pollster::block_on(run());
}
