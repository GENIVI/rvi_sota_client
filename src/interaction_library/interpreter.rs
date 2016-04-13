use std::sync::mpsc::{Sender, Receiver};


pub trait Interpreter<Env, C, E> {

    fn interpret(env: &Env, c: C, e: Sender<E>);

    fn run(env: &Env, rx: Receiver<C>, tx: Sender<E>) {
        loop {
            let c = rx.recv().unwrap();
            Self::interpret(&env, c, tx.clone());
        }
    }

}
