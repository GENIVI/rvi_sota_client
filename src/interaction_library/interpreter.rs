use std::sync::mpsc::{Sender, Receiver};


pub trait Interpreter<Env, C, E> {

    fn interpret(env: &Env, c: C, e: Sender<E>);

    fn run(env: &Env, rx: Receiver<C>, tx: Sender<E>) {
        loop {
            match rx.recv() {
                Ok(c) => Self::interpret(&env, c, tx.clone()),
                Err(e) => error!("Error receiving command: {:?}", e)
            }
        }
    }

}
