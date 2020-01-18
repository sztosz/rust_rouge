rltk::add_wasm_support!();
use rltk::{Rltk, GameState, Console};

struct State {}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "RLTK ROUGE");
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "RLTK Rouge", "resources");
    let games_state = State{};
    rltk::main_loop(context, games_state)
}