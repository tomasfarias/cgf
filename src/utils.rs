use log;

use shakmaty::{san::SanPlus, Color, Move, Position, Role, Square};

const ASCII: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!?";

pub fn next_move<P: Position>(moves: &mut Vec<char>, position: &mut P) -> Option<String> {
    if moves.is_empty() {
        return None;
    }

    let start = moves.pop().unwrap();
    let end = moves.pop().unwrap();

    let index_start = ASCII.find(start).unwrap();
    let promo_left = match position.turn() {
        Color::Black => index_start as i8 - 9,
        Color::White => index_start as i8 + 7,
    };
    let promo_right = match position.turn() {
        Color::Black => index_start as i8 - 7,
        Color::White => index_start as i8 + 9,
    };
    let promo_center = match position.turn() {
        Color::Black => index_start as i8 - 8,
        Color::White => index_start as i8 + 8,
    };

    let (index_end, promotion) = match ASCII.find(end) {
        Some(i) => (i, None),
        None => match end {
            '}' => (promo_right as usize, Some(Role::Queen)),
            ')' => (promo_right as usize, Some(Role::Knight)),
            ']' => (promo_right as usize, Some(Role::Rook)),
            '$' => (promo_right as usize, Some(Role::Bishop)),
            '~' => (promo_center as usize, Some(Role::Queen)),
            '^' => (promo_center as usize, Some(Role::Knight)),
            '_' => (promo_center as usize, Some(Role::Rook)),
            '#' => (promo_center as usize, Some(Role::Bishop)),
            '{' => (promo_left as usize, Some(Role::Queen)),
            '(' => (promo_left as usize, Some(Role::Knight)),
            '[' => (promo_left as usize, Some(Role::Rook)),
            '@' => (promo_left as usize, Some(Role::Bishop)),
            _ => panic!("well crap"),
        },
    };

    let square_start = Square::new(index_start as u32);
    let square_end = Square::new(index_end as u32);
    log::debug!("Squares: {}, {}", square_start, square_end);

    let piece_end_role = match position.board().piece_at(square_end) {
        Some(piece) => Some(piece.role),
        None => None,
    };
    let piece_start = position.board().piece_at(square_start).unwrap().role;

    let current_color = position.turn();

    let m = match piece_start {
        Role::King => {
            if i8::abs(index_start as i8 - index_end as i8) > 1
                && square_end.rank() == square_start.rank()
            {
                // Only instance when king moves more than 1 square is castle
                let rook_square = match (current_color, index_start as i8 - index_end as i8) {
                    (Color::Black, -2) => Square::new(63),
                    (Color::Black, 2) => Square::new(56),
                    (Color::White, -2) => Square::new(7),
                    (Color::White, 2) => Square::new(0),
                    _ => panic!("well crap"),
                };
                Move::Castle {
                    king: square_start,
                    rook: rook_square,
                }
            } else {
                Move::Normal {
                    role: piece_start,
                    from: square_start,
                    capture: piece_end_role,
                    to: square_end,
                    promotion: None,
                }
            }
        }
        _ => Move::Normal {
            role: piece_start,
            from: square_start,
            capture: piece_end_role,
            to: square_end,
            promotion: promotion,
        },
    };
    log::debug!("Move: {:?}", m);

    let sanplus = SanPlus::from_move_and_play_unchecked(position, &m);
    Some(format!("{}", sanplus))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{fen::Fen, CastlingMode, Chess};

    #[test]
    fn test_next_move_e4_e5() {
        let mut moves: Vec<char> = vec!['K', '0', 'C', 'm'];
        let mut position = Chess::default();

        let e4 = next_move(&mut moves, &mut position);
        assert_eq!(e4, Some("e4".to_string()));
        assert_eq!(moves, vec!['K', '0']);

        let e5 = next_move(&mut moves, &mut position);
        assert_eq!(e5, Some("e5".to_string()));
        assert_eq!(moves, Vec::<char>::new());

        let no_moves = next_move(&mut moves, &mut position);
        assert_eq!(no_moves, None);
    }

    #[test]
    fn test_next_move_e4_captures() {
        let mut moves: Vec<char> = vec!['J', 'C', 'J', 'Z', 'C', 'm'];
        let mut position = Chess::default();

        let e4 = next_move(&mut moves, &mut position);
        assert_eq!(e4, Some("e4".to_string()));
        assert_eq!(moves, vec!['J', 'C', 'J', 'Z']);

        let d5 = next_move(&mut moves, &mut position);
        assert_eq!(d5, Some("d5".to_string()));
        assert_eq!(moves, vec!['J', 'C']);

        let exd5 = next_move(&mut moves, &mut position);
        assert_eq!(exd5, Some("exd5".to_string()));
        assert_eq!(moves, Vec::<char>::new());

        let no_moves = next_move(&mut moves, &mut position);
        assert_eq!(no_moves, None);
    }

    #[test]
    fn test_next_move_bongcloud() {
        let mut moves: Vec<char> = vec!['B', '7', 'u', 'm', 'C', 'J', 'm', 'e', 'J', 'Z', 'C', 'm'];
        let mut position = Chess::default();

        let e4 = next_move(&mut moves, &mut position);
        assert_eq!(e4, Some("e4".to_string()));
        assert_eq!(
            moves,
            vec!['B', '7', 'u', 'm', 'C', 'J', 'm', 'e', 'J', 'Z']
        );

        let d5 = next_move(&mut moves, &mut position);
        assert_eq!(d5, Some("d5".to_string()));
        assert_eq!(moves, vec!['B', '7', 'u', 'm', 'C', 'J', 'm', 'e']);

        let ke2 = next_move(&mut moves, &mut position);
        assert_eq!(ke2, Some("Ke2".to_string()));
        assert_eq!(moves, vec!['B', '7', 'u', 'm', 'C', 'J']);

        let dxe4 = next_move(&mut moves, &mut position);
        assert_eq!(dxe4, Some("dxe4".to_string()));
        assert_eq!(moves, vec!['B', '7', 'u', 'm']);

        let ke3 = next_move(&mut moves, &mut position);
        assert_eq!(ke3, Some("Ke3".to_string()));
        assert_eq!(moves, vec!['B', '7']);

        let qd4 = next_move(&mut moves, &mut position);
        assert_eq!(qd4, Some("Qd4+".to_string()));
        assert_eq!(moves, Vec::<char>::new());
        assert_eq!(position.is_check(), true);

        let no_moves = next_move(&mut moves, &mut position);
        assert_eq!(no_moves, None);
    }

    #[test]
    fn test_next_move_scholars_mate() {
        let mut moves: Vec<char> = vec![
            '1', 'N', 'T', '!', 'A', 'f', 'Q', '5', 'N', 'd', 'K', '0', 'C', 'm',
        ];
        let mut position = Chess::default();

        let e4 = next_move(&mut moves, &mut position);
        assert_eq!(e4, Some("e4".to_string()));
        assert_eq!(
            moves,
            vec!['1', 'N', 'T', '!', 'A', 'f', 'Q', '5', 'N', 'd', 'K', '0']
        );

        let e5 = next_move(&mut moves, &mut position);
        assert_eq!(e5, Some("e5".to_string()));
        assert_eq!(
            moves,
            vec!['1', 'N', 'T', '!', 'A', 'f', 'Q', '5', 'N', 'd']
        );

        let qh5 = next_move(&mut moves, &mut position);
        assert_eq!(qh5, Some("Qh5".to_string()));
        assert_eq!(moves, vec!['1', 'N', 'T', '!', 'A', 'f', 'Q', '5']);

        let nc6 = next_move(&mut moves, &mut position);
        assert_eq!(nc6, Some("Nc6".to_string()));
        assert_eq!(moves, vec!['1', 'N', 'T', '!', 'A', 'f']);

        let bc4 = next_move(&mut moves, &mut position);
        assert_eq!(bc4, Some("Bc4".to_string()));
        assert_eq!(moves, vec!['1', 'N', 'T', '!']);

        let nf6 = next_move(&mut moves, &mut position);
        assert_eq!(nf6, Some("Nf6".to_string()));
        assert_eq!(moves, vec!['1', 'N']);

        let qxf7 = next_move(&mut moves, &mut position);
        assert_eq!(qxf7, Some("Qxf7#".to_string()));
        assert_eq!(moves, Vec::<char>::new());
        assert_eq!(position.is_checkmate(), true);

        let no_moves = next_move(&mut moves, &mut position);
        assert_eq!(no_moves, None);
    }

    #[test]
    fn test_next_move_castle_king_side() {
        let mut moves: Vec<char> = vec!['g', 'e'];
        let fen_str = b"rnb1kbnr/ppp2ppp/3p4/4p1q1/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4";
        let mut position: Chess = Fen::from_ascii(fen_str)
            .unwrap()
            .position(CastlingMode::Standard)
            .unwrap();

        let castle = next_move(&mut moves, &mut position);
        assert_eq!(castle, Some("O-O".to_string()));
    }

    #[test]
    fn test_next_move_castle_queen_side() {
        let mut moves: Vec<char> = vec!['6', '8'];
        let fen_str = b"r3kbnr/p1pp1ppp/bpn5/4p1q1/2B1P3/3P1N2/PPP1QPPP/RNB2RK1 b kq - 0 6";
        let mut position: Chess = Fen::from_ascii(fen_str)
            .unwrap()
            .position(CastlingMode::Standard)
            .unwrap();

        let castle = next_move(&mut moves, &mut position);
        assert_eq!(castle, Some("O-O-O".to_string()));
    }

    #[test]
    fn test_next_move_promote_to_queen() {
        let mut moves: Vec<char> = vec!['}', 'm'];
        let fen_str = b"2kr1bnr/p1p3pp/bpn5/4p3/4P3/5P1N/PPP1p1PP/RNB2RK1 b - - 0 11";
        let mut position: Chess = Fen::from_ascii(fen_str)
            .unwrap()
            .position(CastlingMode::Standard)
            .unwrap();

        let castle = next_move(&mut moves, &mut position);
        assert_eq!(castle, Some("exf1=Q#".to_string()));
    }

    #[test]
    fn test_next_move_promote_to_knight() {
        let mut moves: Vec<char> = vec!['^', 'm'];
        let fen_str = b"2kr1bnr/p1p3pp/bpn5/4p3/4P3/5P1N/PPP1p1PP/RNB2RK1 b - - 0 11";
        let mut position: Chess = Fen::from_ascii(fen_str)
            .unwrap()
            .position(CastlingMode::Standard)
            .unwrap();

        let castle = next_move(&mut moves, &mut position);
        assert_eq!(castle, Some("e1=N".to_string()));
    }
}
