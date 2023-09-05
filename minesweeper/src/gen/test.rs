use super::*;

fn place_mine(game: &mut Game, x: i16, y: i16) {
    game[(x, y)].state = FieldState::Mine;
    game.increment_field(x - 1, y - 1);
    game.increment_field(x - 1, y + 0);
    game.increment_field(x - 1, y + 1);
    game.increment_field(x + 0, y - 1);
    game.increment_field(x + 0, y + 1);
    game.increment_field(x + 1, y - 1);
    game.increment_field(x + 1, y + 0);
    game.increment_field(x + 1, y + 1);
}

fn game(width: i16, height: i16) -> Game {
    Game::new(width, height, 0.0..1.0, crate::Difficulty::Easy, false)
}

#[test]
fn ambigous_board() {
    let mut game = game(5, 5);
    place_mine(&mut game, 3, 1);
    place_mine(&mut game, 2, 2);
    place_mine(&mut game, 1, 3);

    let res = game.validate_board(0, 0);
    assert_eq!(res, Err(Error::Ambigous));
}

#[test]
fn solvable_board_1() {
    let mut game = game(5, 5);
    place_mine(&mut game, 2, 2);
    place_mine(&mut game, 2, 3);

    let res = game.validate_board(0, 0);
    assert_eq!(res, Ok(()));
}

#[test]
fn solvable_board_2() {
    let mut game = game(4, 5);
    place_mine(&mut game, 0, 3);
    place_mine(&mut game, 1, 2);
    place_mine(&mut game, 2, 2);
    place_mine(&mut game, 0, 4);

    let res = game.validate_board(0, 0);
    assert_eq!(res, Ok(()));
}

#[test]
fn solvable_board_3() {
    let mut game = game(9, 5);
    place_mine(&mut game, 0, 3);
    place_mine(&mut game, 1, 2);
    place_mine(&mut game, 2, 2);
    place_mine(&mut game, 4, 2);
    place_mine(&mut game, 6, 2);
    place_mine(&mut game, 7, 2);
    place_mine(&mut game, 8, 3);

    let res = game.validate_board(0, 0);
    assert_eq!(res, Ok(()));
}

#[test]
fn solvable_board_4() {
    let mut game = game(5, 5);
    place_mine(&mut game, 2, 2);
    place_mine(&mut game, 1, 3);

    let res = game.validate_board(0, 0);
    assert_eq!(res, Ok(()));
}

#[test]
fn hidden_adjacents_1() {
    let game = game(5, 5);

    let hidden_adjacents = game.hidden_adjacents(0, 0);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((1, 0));
    expected.push((1, 1));
    expected.push((0, 1));
    assert_eq!(values, expected);
}

#[test]
fn hidden_adjacents_2() {
    let mut game = game(5, 5);
    game[(1, 1)].visibility = Visibility::Hint;

    let hidden_adjacents = game.hidden_adjacents(0, 0);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((1, 0));
    expected.push((0, 1));
    assert_eq!(values, expected);
}

#[test]
fn hidden_adjacents_3() {
    let game = game(5, 5);

    let hidden_adjacents = game.hidden_adjacents(4, 0);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((0, 1));
    expected.push((-1, 1));
    expected.push((-1, 0));
    assert_eq!(values, expected);
}

#[test]
fn hidden_adjacents_4() {
    let mut game = game(5, 5);
    game[(3, 1)].visibility = Visibility::Hint;

    let hidden_adjacents = game.hidden_adjacents(4, 0);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((0, 1));
    expected.push((-1, 0));
    assert_eq!(values, expected);
}

#[test]
fn hidden_adjacents_5() {
    let game = game(5, 5);

    let hidden_adjacents = game.hidden_adjacents(4, 4);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((-1, -1));
    expected.push((0, -1));
    expected.push((-1, 0));
    assert_eq!(values, expected);
}

#[test]
fn hidden_adjacents_6() {
    let mut game = game(5, 5);
    game[(3, 3)].visibility = Visibility::Hint;

    let hidden_adjacents = game.hidden_adjacents(4, 4);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((0, -1));
    expected.push((-1, 0));
    assert_eq!(values, expected);
}

#[test]
fn hidden_adjacents_7() {
    let game = game(5, 5);

    let hidden_adjacents = game.hidden_adjacents(0, 4);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((0, -1));
    expected.push((1, -1));
    expected.push((1, 0));
    assert_eq!(values, expected);
}

#[test]
fn hidden_adjacents_8() {
    let mut game = game(5, 5);
    game[(1, 3)].visibility = Visibility::Hint;

    let hidden_adjacents = game.hidden_adjacents(0, 4);
    let values = hidden_adjacents.offsets();

    let mut expected = StackVec::new();
    expected.push((0, -1));
    expected.push((1, 0));
    assert_eq!(values, expected);
}
