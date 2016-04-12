use std::sync::mpsc::{Sender, Receiver};


pub trait Interpreter<Env: Default, C, E> {

    fn interpret(env: &Env, c: C, e: Sender<E>);

    fn run_interpreter(rx: Receiver<C>, tx: Sender<E>) {

        let env = Env::default();

        loop {
            let c = rx.recv().unwrap();
            Self::interpret(&env, c, tx.clone());
        }

    }

}
