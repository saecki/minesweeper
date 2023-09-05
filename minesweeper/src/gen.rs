use rand::Rng;

use crate::combination_iter::CombinationIter;
use crate::stackvec::StackVec;
use crate::{FieldState, Game, Visibility};

#[cfg(test)]
mod test;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Error {
    Invalid,
    Ambigous,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid => f.write_str("Invalid"),
            Self::Ambigous => f.write_str("Ambigous"),
        }
    }
}

impl std::error::Error for Error {}

fn print_game(game: &Game, mx: i16, my: i16, n: u8) {
    let mut buf = String::new();
    fmt_game(&mut buf, game, mx, my, n).unwrap();
    println!("{buf}");
}

fn fmt_game(
    f: &mut impl std::fmt::Write,
    game: &Game,
    mx: i16,
    my: i16,
    n: u8,
) -> std::fmt::Result {
    write!(f, "{:1$}", "", n as usize * 4)?;
    write!(f, "  ")?;
    for x in 0..game.width {
        write!(f, "{x:2}")?;
    }
    writeln!(f)?;

    for y in 0..game.height {
        write!(f, "{:1$}", "", n as usize * 4)?;
        write!(f, "{y:2}")?;
        for x in 0..game.width {
            let field = &game[(x, y)];
            if x == mx && y == my {
                write!(f, "\x1b[1;7;34m")?;
            } else {
                match field.visibility {
                    Visibility::Hide => write!(f, "\x1b[1;7;90m")?,
                    Visibility::Hint => write!(f, "\x1b[1;7;33m")?,
                    Visibility::Show => write!(f, "\x1b[1;7;92m")?,
                };
            }
            match field.state {
                FieldState::Free(n) if n == 0 => write!(f, "  ")?,
                FieldState::Free(n) => write!(f, " {n}")?,
                FieldState::Mine => write!(f, " *")?,
            }
            write!(f, "\x1b[0m")?;
        }
        writeln!(f)?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
enum Solve {
    Progress(Game),
    NoMissingNeighbors,
    Done,
}

impl Game {
    pub fn is_solved(&self) -> bool {
        for f in self.fields.iter() {
            if let FieldState::Free(_) = f.state {
                if f.visibility != Visibility::Show {
                    return false;
                }
            }
        }

        true
    }

    pub fn gen_board(&mut self) {
        let mut rng = rand::thread_rng();
        let mut available_indices = self.fields.len();

        for _ in 0..self.num_mines {
            let mut available_idx = rng.gen_range(0..available_indices);
            for (actual_index, f) in self.fields.iter_mut().enumerate() {
                if f.state != FieldState::Mine {
                    if available_idx == 0 {
                        f.state = FieldState::Mine;

                        let x = (actual_index % self.width as usize) as i16;
                        let y = (actual_index / self.width as usize) as i16;

                        self.increment_field(x - 1, y - 1);
                        self.increment_field(x - 1, y + 0);
                        self.increment_field(x - 1, y + 1);
                        self.increment_field(x + 0, y - 1);
                        self.increment_field(x + 0, y + 1);
                        self.increment_field(x + 1, y - 1);
                        self.increment_field(x + 1, y + 0);
                        self.increment_field(x + 1, y + 1);
                        break;
                    }
                    available_idx -= 1;
                }
            }

            available_indices -= 1;
        }
    }

    pub fn is_unambigous(&self, x: i16, y: i16) -> bool {
        let mut board = self.clone();
        board.validate_board(x, y) == Ok(())
    }

    /// Try to validate a board by:
    /// 1. Try to solve as far as possible using these simple techniques:
    ///     1. When the number of hidden fields equals the number of neighbors of a visible field -> place hints on them
    ///     2. When the number of hinted fields equals the number of neighbors -> show the other fields
    /// 2. When no more of the above rules are applicable pick a field that has more hidden fields
    ///    surrounding them than neighbors. We're now at a fork:
    ///     - Try out all possible solutions
    ///     - The board is ambiguous If just one of the resulting solutions is valid and others are determined to be
    ///       invalid because:
    ///         1. A field has more hints surrounding them than the number of neighbors
    ///     - If all resulting solutions are valid, the generated board is ambiguous. One of them
    ///       will have a free field that has a hint on it. Or a mine was shown.
    fn validate_board(&mut self, x: i16, y: i16) -> Result<(), Error> {
        let mut board = self.clone();

        loop {
            board.solve_board(x, y, true)?;
            if board.is_solved() {
                return Ok(());
            }

            let mut copy = board.clone();
            loop {
                for y in 0..board.height {
                    for x in 0..board.width {
                        if board[(x, y)].visibility == Visibility::Show {
                            board.solve_board(x, y, true)?;
                            if board.is_solved() {
                                return Ok(());
                            }
                        }
                    }
                }

                if copy == board {
                    break;
                }

                copy.clone_from(&board);
            }

            match board.guess_mines(0, board.width, 0, board.height, 0) {
                Err(e) => return Err(e),
                Ok(Solve::Done) => return Ok(()),
                Ok(Solve::Progress(b)) => board = b,
                Ok(Solve::NoMissingNeighbors) => return Err(Error::Ambigous),
            }
        }
    }

    fn guess_mines(&self, x_s: i16, x_e: i16, y_s: i16, y_e: i16, n: u8) -> Result<Solve, Error> {
        let mut possible_fields = Vec::new();
        for y in y_s..y_e {
            for x in x_s..x_e {
                let field = self[(x, y)];
                if field.visibility == Visibility::Show {
                    if let FieldState::Free(neighbors) = field.state {
                        let hidden_adjacents = self.hidden_adjacents(x, y);
                        let hinted_adjacents = self.hinted_adjacents(x, y);
                        let num_missing_neighbors = neighbors - hinted_adjacents.num();

                        if num_missing_neighbors > 0
                            && num_missing_neighbors < hidden_adjacents.num()
                        {
                            possible_fields.push((x, y, num_missing_neighbors, hidden_adjacents));
                        }
                    }
                }
            }
        }
        if possible_fields.len() == 0 {
            println!("no missing neighbors");
            return Ok(Solve::NoMissingNeighbors);
        }

        possible_fields.sort_unstable_by(|(_, _, n1, a1), (_, _, n2, a2)| {
            Ord::cmp(&(a1.num() - n1), &(a2.num() - n2)).then(Ord::cmp(&n1, &n2))
        });

        let mut num_ambigous = 0;
        'guessing: for &(x, y, num_missing_neighbors, adjacents) in possible_fields.iter() {
            if self.open_mine_count() < num_missing_neighbors as i16 {
                // The board is invalid, some hints have been placed incorrectly.
                println!("invalid minecount");
                return Err(Error::Invalid);
            }

            let num_hidden = adjacents.num();
            let offsets = adjacents.offsets();

            let iter = CombinationIter::new(num_hidden, num_missing_neighbors);
            let mut valid_board = None;
            'combinations: for combination in iter {
                let mut board = self.clone();
                for fi in 0..num_hidden {
                    if combination[fi as usize] {
                        let (x_off, y_off) = offsets[fi as usize];
                        board[(x + x_off, y + y_off)].visibility = Visibility::Hint;
                    }
                }

                // check if the board is actually still valid, or if these guesses are already
                // invalid
                let x_s = i16::max(x - 2, 0);
                let x_e = i16::min(x + 3, board.width);
                let y_s = i16::max(y - 2, 0);
                let y_e = i16::min(y + 3, board.height);
                for fy in y_s..y_e {
                    for fx in x_s..x_e {
                        let field = board[(fx, fy)];
                        if field.visibility == Visibility::Show {
                            if let FieldState::Free(neighbors) = field.state {
                                let hinted_adjacents = board.hinted_adjacents(fx, fy);
                                if hinted_adjacents.num() > neighbors {
                                    println!("invalid other");
                                    continue 'combinations;
                                }
                            }
                        }
                    }
                }

                if board.open_mine_count() == 0 {
                    // If there are no mines left there should be no missing neighbors
                    for y in 0..board.height {
                        for x in 0..board.width {
                            if !board.is_in_bounds(x, y) {
                                continue;
                            }

                            let field = board[(x, y)];
                            if field.visibility == Visibility::Show {
                                if let FieldState::Free(neighbors) = field.state {
                                    let hinted_adjacents = board.hinted_adjacents(x, y);
                                    if hinted_adjacents.num() < neighbors {
                                        println!("invalid still open fields");
                                        continue 'combinations;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let x_s = i16::max(x - 3, 0);
                    let x_e = i16::min(x + 4, board.width);
                    let y_s = i16::max(y - 3, 0);
                    let y_e = i16::min(y + 4, board.height);
                    match board.guess_mines(x_s, x_e, y_s, y_e, n + 1) {
                        Err(Error::Invalid) => continue 'combinations,
                        Err(Error::Ambigous) => {
                            // later step are ambigous but if all other combinations are invalid,
                            // everything up to here has to be right.
                        }
                        Ok(Solve::Done) => return Ok(Solve::Done),
                        Ok(Solve::Progress(b)) => board = b,
                        Ok(Solve::NoMissingNeighbors) => (),
                    }
                }

                if valid_board.is_none() {
                    valid_board = Some(board);
                } else {
                    println!("ambigous");
                    num_ambigous += 1;
                    continue 'guessing;
                }
            }

            if let Some(valid_board) = valid_board {
                // println!("exactly one found");
                if valid_board.is_solved() {
                    println!("solved");
                    return Ok(Solve::Done);
                }

                // Lock in the progress and repeat steps
                if n == 1 {
                    println!("progress with:");
                    print_game(&valid_board, -1, -1, n);
                }
                return Ok(Solve::Progress(valid_board));
            }
        }

        if num_ambigous > 0 {
            println!("{num_ambigous} ambigous");
            Err(Error::Ambigous)
        } else {
            println!("just invalid");
            Err(Error::Invalid)
        }
    }

    fn solve_board(&mut self, x: i16, y: i16, force: bool) -> Result<(), Error> {
        if !self.is_in_bounds(x, y) {
            return Ok(());
        }

        let field = &mut self[(x, y)];
        match field.visibility {
            Visibility::Hide => {
                if field.state == FieldState::Mine {
                    return Err(Error::Invalid);
                }
                field.visibility = Visibility::Show;
            }
            Visibility::Hint => return Ok(()),
            Visibility::Show if force => (),
            Visibility::Show => return Ok(()),
        }

        match field.state {
            FieldState::Free(0) => {
                self.solve_board(x - 1, y - 1, false)?;
                self.solve_board(x + 0, y - 1, false)?;
                self.solve_board(x + 1, y - 1, false)?;
                self.solve_board(x - 1, y + 0, false)?;
                self.solve_board(x + 1, y + 0, false)?;
                self.solve_board(x - 1, y + 1, false)?;
                self.solve_board(x + 0, y + 1, false)?;
                self.solve_board(x + 1, y + 1, false)?;
                Ok(())
            }
            FieldState::Free(neighbors) => {
                let hidden_adjacents = self.hidden_adjacents(x, y);
                let hinted_adjacents = self.hinted_adjacents(x, y);
                let num_missing_neighbors = neighbors - hinted_adjacents.num();
                if num_missing_neighbors == hidden_adjacents.num() {
                    self.hint_hidden_field(x - 1, y - 1);
                    self.hint_hidden_field(x - 1, y + 0);
                    self.hint_hidden_field(x - 1, y + 1);
                    self.hint_hidden_field(x + 0, y - 1);
                    self.hint_hidden_field(x + 0, y + 1);
                    self.hint_hidden_field(x + 1, y - 1);
                    self.hint_hidden_field(x + 1, y + 0);
                    self.hint_hidden_field(x + 1, y + 1);
                }

                let hinted_adjacents = self.hinted_adjacents(x, y);
                if neighbors == hinted_adjacents.num() {
                    self.solve_board(x - 1, y - 1, false)?;
                    self.solve_board(x - 1, y + 0, false)?;
                    self.solve_board(x - 1, y + 1, false)?;
                    self.solve_board(x + 0, y - 1, false)?;
                    self.solve_board(x + 0, y + 1, false)?;
                    self.solve_board(x + 1, y - 1, false)?;
                    self.solve_board(x + 1, y + 0, false)?;
                    self.solve_board(x + 1, y + 1, false)?;
                }
                Ok(())
            }
            FieldState::Mine => Err(Error::Invalid),
        }
    }

    fn hint_hidden_field(&mut self, x: i16, y: i16) {
        if !self.is_in_bounds(x, y) {
            return;
        }

        let field = &mut self[(x, y)];

        if field.visibility == Visibility::Hide {
            field.visibility = Visibility::Hint;
        }
    }

    fn increment_field(&mut self, x: i16, y: i16) {
        if self.is_in_bounds(x, y) {
            if let FieldState::Free(neighbors) = &mut self[(x, y)].state {
                *neighbors += 1;
            }
        }
    }

    pub fn hinted_adjacents(&self, x: i16, y: i16) -> Adjacents {
        Adjacents::new(
            self.is_hinted_field(x - 1, y - 1),
            self.is_hinted_field(x + 0, y - 1),
            self.is_hinted_field(x + 1, y - 1),
            self.is_hinted_field(x + 1, y + 0),
            self.is_hinted_field(x + 1, y + 1),
            self.is_hinted_field(x + 0, y + 1),
            self.is_hinted_field(x - 1, y + 1),
            self.is_hinted_field(x - 1, y + 0),
        )
    }

    fn is_hinted_field(&self, x: i16, y: i16) -> bool {
        if !self.is_in_bounds(x, y) {
            return false;
        }

        self[(x, y)].visibility == Visibility::Hint
    }

    pub fn hidden_adjacents(&self, x: i16, y: i16) -> Adjacents {
        Adjacents::new(
            self.is_hidden_field(x - 1, y - 1),
            self.is_hidden_field(x + 0, y - 1),
            self.is_hidden_field(x + 1, y - 1),
            self.is_hidden_field(x + 1, y + 0),
            self.is_hidden_field(x + 1, y + 1),
            self.is_hidden_field(x + 0, y + 1),
            self.is_hidden_field(x - 1, y + 1),
            self.is_hidden_field(x - 1, y + 0),
        )
    }

    fn is_hidden_field(&self, x: i16, y: i16) -> bool {
        if !self.is_in_bounds(x, y) {
            return false;
        }

        self[(x, y)].visibility == Visibility::Hide
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adjacents(u8);

impl Adjacents {
    const NW: u8 = 0x01;
    const N: u8 = 0x02;
    const NE: u8 = 0x04;
    const E: u8 = 0x08;
    const SE: u8 = 0x10;
    const S: u8 = 0x20;
    const SW: u8 = 0x40;
    const W: u8 = 0x80;

    pub fn new(nw: bool, n: bool, ne: bool, e: bool, se: bool, s: bool, sw: bool, w: bool) -> Self {
        Self(
            nw.then_some(Self::NW).unwrap_or(0)
                | n.then_some(Self::N).unwrap_or(0)
                | ne.then_some(Self::NE).unwrap_or(0)
                | e.then_some(Self::E).unwrap_or(0)
                | se.then_some(Self::SE).unwrap_or(0)
                | s.then_some(Self::S).unwrap_or(0)
                | sw.then_some(Self::SW).unwrap_or(0)
                | w.then_some(Self::W).unwrap_or(0),
        )
    }

    pub fn num(&self) -> u8 {
        self.0.count_ones() as u8
    }

    fn offsets(&self) -> StackVec<8, (i16, i16)> {
        let mut offsets = StackVec::new();
        if self.0 & Self::NW != 0 {
            offsets.push((-1, -1))
        }
        if self.0 & Self::N != 0 {
            offsets.push((0, -1))
        }
        if self.0 & Self::NE != 0 {
            offsets.push((1, -1))
        }
        if self.0 & Self::E != 0 {
            offsets.push((1, 0))
        }
        if self.0 & Self::SE != 0 {
            offsets.push((1, 1))
        }
        if self.0 & Self::S != 0 {
            offsets.push((0, 1))
        }
        if self.0 & Self::SW != 0 {
            offsets.push((-1, 1))
        }
        if self.0 & Self::W != 0 {
            offsets.push((-1, 0))
        }

        offsets
    }
}
