mod game;

use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::windows::{WindowInstance, WindowInstanceProps};

pub fn new_tictactoe_instance() -> WindowInstance {
    WindowInstance::new(WindowInstanceProps {
        title: "tictactoe".to_owned(),
        resizable: false,
        size: ScreenCoordinates::Absolute { x: 218, y: 268 },
        ..Default::default()
    }, move |_| rsx! {
        WindowTicTacToe {}
    })
}

#[component]
fn WindowTicTacToe() -> Element {
    let mut board = use_signal(|| game::Board::new(game::Difficulty::Easy));
    
    rsx! {
        div {
            class: "tictactoe",
            
            div {
                class: "ttt-difficulty-selector",
                for difficulty in game::Difficulty::all() {
                    button {
                        class: if *difficulty == board.read().difficulty {
                            "difficulty-button selected"
                        } else {
                            "difficulty-button"
                        },
                        id: match *difficulty {
                            game::Difficulty::Easy => "easy",
                            game::Difficulty::Difficult => "difficult",
                            game::Difficulty::GoodLuck => "goodluck",
                        },
                        onclick: move |_| {
                            board.write().difficulty = *difficulty;
                            board.write().reset();
                        },
                        {difficulty.to_string()}
                    }
                }
            }
            
            div {
                class: "ttt-board",
                
                if board.read().game_over() {
                    div {
                        class: "ttt-gameover-overlay",
                        div {
                            class: "ttt-gameover-text",
                            match board.read().calculate_winner() {
                                Some(game::Cell::X) => "You win!",
                                Some(game::Cell::O) => "You lost!",
                                _ => "Draw!",
                            }
                        }
                        button {
                            class: "ttt-play-again-button",
                            onclick: move |_| {
                                board.write().reset();
                            },
                            "Play Again"
                        }
                    }
                }
                
                for (i, cell) in board.read().cells.iter().enumerate() {
                    div {
                        class: "cell",
                        id: {
                            match cell {
                                game::Cell::Empty => "empty".to_owned(),
                                game::Cell::X => "x".to_owned(),
                                game::Cell::O => "o".to_owned(),
                            }
                        },
                        onclick: move |_| {
                            board.write().play(i as u8);
                        },
                        {cell.to_string()}
                    }
                }
            }
        }
    }
}