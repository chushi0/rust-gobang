use dialoguer::Input;
use game::GameState;
use std::time::SystemTime;

use crate::game::GameEnd;

pub mod ai;
pub mod game;

fn main() {
    let mut game_state = GameState::new();
    let mut turn = 0;

    loop {
        println!("========================");
        println!("Turn: {turn}");
        println!("{game_state}");

        if turn % 2 == 0 {
            let x = Input::new()
                .with_prompt("x")
                .interact_text()
                .expect("input x");
            let y = Input::new()
                .with_prompt("y")
                .interact_text()
                .expect("input y");

            match game_state.make_move((x, y)) {
                Ok(GameEnd::Win) => {
                    println!("You WIN")
                }
                Ok(GameEnd::Lost) => {
                    println!("You Lost")
                }
                Ok(GameEnd::NotEnd) => {}
                Err(err) => {
                    println!("Move fail: {err}");
                    continue;
                }
            }
        } else {
            let start_time = SystemTime::now();
            let (point, score) = ai::best_move(&game_state);
            let point = point.expect("AI should always move one");
            let end_time = SystemTime::now();
            println!("AI Move: {point:?}");
            println!("AI Score: {score:?}");
            println!(
                "AI Cost Time: {} ms",
                end_time
                    .duration_since(start_time)
                    .expect("should valid")
                    .as_millis()
            );
            match game_state.make_move(point).expect("AI move fail") {
                GameEnd::Win => {
                    println!("AI WIN")
                }
                GameEnd::Lost => {
                    println!("AI Lost")
                }
                GameEnd::NotEnd => {}
            }
        }

        turn += 1;
    }
}
