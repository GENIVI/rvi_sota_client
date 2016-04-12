pub use self::console::Console;
pub use self::gateway::Gateway;
pub use self::interpreter::Interpreter;
pub use self::parse::Parse;
pub use self::print::Print;

mod broadcast;
pub mod console;
pub mod gateway;
pub mod interpreter;
pub mod parse;
pub mod print;
pub mod websocket;


#[macro_export]
macro_rules! interact {
    ( $( $g: ident ), * ) => {
        {
            let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
            let (ev_tx,  ev_rx)  = std::sync::mpsc::channel();

            let mut broadcast = broadcast::Broadcast::new(ev_rx);

            $(
                $g::run(cmd_tx.clone(), broadcast.subscribe());
            )*

            std::thread::spawn(move || broadcast.start());

            run_interpreter(cmd_rx, ev_tx);
        }

    };
}
