use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use std::thread;


pub trait Gateway<C, E>: Sized + Send + Sync + 'static
    where
    C: Send + 'static, E: Send + 'static {

    fn new() -> Self;
    fn get_line(&self) -> String;
    fn put_line(&self, s: String);

    fn parse(s: String) -> Option<C>;
    fn pretty_print(e: E) -> String;

    fn run(tx: Sender<C>, rx: Receiver<E>) {

        let io = Arc::new(Self::new());

        // Read lines.
        let io_clone = io.clone();

        thread::spawn(move || {
            loop {
                let cmd = Self::parse(io_clone.get_line()).unwrap();
                tx.send(cmd).unwrap()
            }
        });

        // Put lines.
        thread::spawn(move || {
            loop {
                let e = rx.recv().unwrap();
                io.put_line(Self::pretty_print(e));
            }
        });

    }

}
