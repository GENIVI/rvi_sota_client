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
                let _ = Self::parse(io_clone.get_line())
                    .ok_or_else(|| error!("Error parsing command"))
                    .and_then(|cmd| {
                        tx.send(cmd).map_err(|e| error!("Error forwarding command: {:?}", e))
                    });
            }
        });

        // Put lines.
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(e) => io.put_line(Self::pretty_print(e)),
                    Err(err) => error!("Error receiving event: {:?}", err)
                }
            }
        });

    }

}
