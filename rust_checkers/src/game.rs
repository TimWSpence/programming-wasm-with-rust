use super::board::{Coordinate, GamePiece, PieceColour, Move};

pub struct GameEngine {
    board: [[Option<GamePiece>; 8]; 8],
    current_turn: PieceColour,
    move_count: u32
}


pub struct MoveResult {
    pub mv: Move,
    pub crowned: bool
}

impl GameEngine {
    pub fn new() -> GameEngine {
        let mut engine = GameEngine {
            board: [[None; 8]; 8],
            current_turn: PieceColour::Black,
            move_count: 0
        };
        engine.initialize_pieces();
        engine
    }

    pub fn initialize_pieces(&mut self) {
        [1,3,5,7,0,2,4,6,1,3,5,7]
            .iter()
            .zip([0,0,0,0,1,1,1,1,2,2,2,2].iter())
            .map(|(a,b)| (*a as usize, *b as usize))
            .for_each(|(x,y)| {
                self.board[x][y] = Some(GamePiece::new(PieceColour::White))
            });

        [0,2,4,6,1,3,5,7,0,2,4,6]
        .iter()
        .zip([5,5,5,5,6,6,6,6,7,7,7,7].iter())
        .map(|(a,b)| (*a as usize, *b as usize))
        .for_each(|(x,y)| {
            self.board[x][y] = Some(GamePiece::new(PieceColour::Black))
        });
    }

    pub fn move_piece(&mut self, mv: &Move) -> Result<MoveResult, ()> {
        let legal_moves = self.legal_moves();

        if !legal_moves.contains(mv) {
            return Err(());
        }

        let Coordinate(fx,fy) = mv.from;
        let Coordinate(tx,ty) = mv.to;

        let piece = self.board[fx][fy].unwrap();
        let midpiece_coordinate = self.midpiece_coordinate(fx,fy,tx,ty);
        if let Some(Coordinate(x,y)) = midpiece_coordinate {
            self.board[x][y] = None; //remove the jumped piece
        }

        //Move piece from source to destination
        self.board[tx][ty] = Some(piece);
        self.board[fx][fy] = None;

        let crowned = if self.should_crown(piece, mv.to) {
            self.crown_piece(mv.to);
            true
        } else {
            false
        };
        self.advance_turn();

        Ok(MoveResult {
            mv: mv.clone(),
            crowned: crowned
        })
    }

    fn legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        for col in 0..8 {
            for row in 0..8 {
                if let Some(piece) = self.board[row][col] {
                    if piece.colour == self.current_turn {
                        let loc = Coordinate(row,col);
                        let mut vmoves = self.valid_moves_from(loc);
                        moves.append(&mut vmoves);
                    }
                }
            }
        }
        moves
    }

    fn valid_moves_from(&self, loc: Coordinate) -> Vec<Move> {
        let Coordinate(x,y) = loc;
        if let Some(p) = self.board[x][y] {
            let mut jumps = loc
                .jump_targets_from()
                .filter(|t| self.valid_jump(&p, &loc, &t))
                .map(|ref t| Move {
                    from: loc.clone(),
                    to: t.clone()
                }).collect::<Vec<Move>>();
            let mut moves = loc
                .move_targets_from()
                .filter(|t| self.valid_move(&p, &loc, &t))
                .map(|ref t| Move {
                    from: loc.clone(),
                    to: t.clone()
                }).collect::<Vec<Move>>();
            jumps.append(&mut moves);
            jumps
        } else {
            Vec::new()
        }
    }

    pub fn get_piece(&self, coord: Coordinate) -> Result<Option<GamePiece>, ()> {
        let Coordinate(x, y) = coord;
        if x <= 7 && y <= 7 {
            Ok(self.board[x][y])
        } else {
            Err(())
        }
    }

    pub fn current_turn(&self) -> PieceColour {
        self.current_turn
    }

    fn advance_turn(&mut self) {
        if self.current_turn == PieceColour::Black {
            self.current_turn = PieceColour::White
        } else {
            self.current_turn = PieceColour::Black
        }
        self.move_count += 1;
    }

    // Black pieces in row 0 or White pieces in row 7 are crowned
    fn should_crown(&self, piece: GamePiece, coord: Coordinate) -> bool {
        let Coordinate(_x, y) = coord;

        (y == 0 && piece.colour == PieceColour::Black) || (y == 7 && piece.colour == PieceColour::White)
    }

    fn crown_piece(&mut self, coord: Coordinate) -> bool {
        let Coordinate(x, y) = coord;
        if let Some(piece) = self.board[x][y] {
            self.board[x][y] = Some(GamePiece::crowned(piece));
            true
        } else {
            false
        }
    }

    pub fn is_crowned(&self, coord: Coordinate) -> bool {
        let Coordinate(x, y) = coord;
        match self.board[x][y] {
            Some(piece) => piece.crowned,
            None => false,
        }
    }

    pub fn move_count(&self) -> u32 {
        self.move_count
    }

    fn midpiece_coordinate(&self, x: usize, y: usize, tx: usize, ty: usize) -> Option<Coordinate> {
        if tx == x + 2 && ty == y + 2 {
            Some(Coordinate(x + 1, y + 1))
        } else if x >= 2 && y >= 2 && tx == x - 2 && ty == y - 2 {
            Some(Coordinate(x - 1, y - 1))
        } else if x >= 2 && tx == x - 2 && ty == y + 2 {
            Some(Coordinate(x - 1, y + 1))
        } else if y >= 2 && tx == x + 2 && ty == y - 2 {
            Some(Coordinate(x + 1, y - 1))
        } else {
            None
        }
    }

    fn midpiece(&self, x: usize, y: usize, tx: usize, ty: usize) -> Option<GamePiece> {
        match self.midpiece_coordinate(x, y, tx, ty) {
            Some(Coordinate(x, y)) => self.board[x][y],
            None => None,
        }
    }

    fn valid_jump(&self, p: &GamePiece, from: &Coordinate, to: &Coordinate) -> bool {
        if !to.on_board() || !from.on_board() {
            false
        } else {
            let Coordinate(x, y) = *from;
            let Coordinate(tx, ty) = *to;

            let midpiece = self.midpiece(x, y, tx, ty);
            match midpiece {
                Some(mp) if mp.colour != p.colour => true,
                _ => false,
            }
        }
    }

    fn valid_move(&self, p: &GamePiece, from: &Coordinate, to: &Coordinate) -> bool {
        if !to.on_board() || !from.on_board() {
            false
        } else {
            let Coordinate(tx, ty) = *to;
            if let Some(_piece) = self.board[tx][ty] {
                false
            } else {
                let Coordinate(_fx, fy) = *from;
                let mut valid = false;
                if ty > fy && p.colour == PieceColour::White {
                    // white moves down
                    valid = true;
                }
                if ty < fy && p.colour == PieceColour::Black {
                    // black moves up
                    valid = true;
                }
                if ty > fy && p.colour == PieceColour::Black && p.crowned {
                    // crowned black mv down
                    valid = true;
                }
                if ty < fy && p.colour == PieceColour::White && p.crowned {
                    // crowned white mv up
                    valid = true;
                }
                valid
            }
        }
    }
}
