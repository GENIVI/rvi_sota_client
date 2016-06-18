use chan;
use chan::{Sender, Receiver};


pub struct Broadcast<A: Clone> {
    peers: Vec<Sender<A>>,
    rx:    Receiver<A>
}

impl<A: Clone> Broadcast<A> {
    pub fn new(rx: Receiver<A>) -> Broadcast<A> {
        Broadcast { peers: vec![], rx: rx }
    }

    pub fn start(&self) {
        loop {
            self.rx.recv().map(|a| {
                for subscriber in &self.peers {
                    subscriber.send(a.clone());
                }
            });
        }
    }

    pub fn subscribe(&mut self) -> Receiver<A> {
        let (tx, rx) = chan::sync::<A>(0);
        self.peers.push(tx);
        rx
    }
}


#[cfg(test)]
mod tests {
    use chan;
    use std::thread;

    use super::*;


    #[test]
    fn test_broadcasts_events() {
        let (tx, rx)      = chan::sync(0);
        let mut broadcast = Broadcast::new(rx);

        let a = broadcast.subscribe();
        let b = broadcast.subscribe();
        thread::spawn(move || broadcast.start());

        tx.send(123);
        assert_eq!(123, a.recv().unwrap());
        assert_eq!(123, b.recv().unwrap());
    }
}
