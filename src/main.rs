use cgf::{ChessError, ChessGameFinder};

fn main() -> Result<(), ChessError> {
    openssl_probe::init_ssl_cert_env_vars();
    env_logger::init();
    let game_finder = ChessGameFinder::new();
    game_finder.run()
}
