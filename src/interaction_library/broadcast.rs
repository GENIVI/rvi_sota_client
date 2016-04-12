use std::sync::mpsc::{Sender, Receiver, channel};


pub struct Broadcast<A: Clone> {
    peers: Vec<Sender<A>>,
    rx: Receiver<A>
}

impl<A: Clone> Broadcast<A> {

    pub fn new(rx: Receiver<A>) -> Broadcast<A> {
        Broadcast { peers: vec![], rx: rx }
    }

    pub fn start(&self) {
        loop {
            match self.rx.recv() {
                Ok(payload) => {
                    for subscriber in &self.peers {
                        match subscriber.send(payload.clone()) {
                            Err(e) => error!("Error broadcasting: {}", e),
                            _ => {}
                        }
                    }
                },
                Err(e) => error!("Error receiving: {}", e)
            }
        }
    }

    pub fn subscribe(&mut self) -> Receiver<A> {
        let (tx, rx) = channel();
        self.peers.push(tx);
        return rx
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc::channel;
    use std::thread::spawn;

    #[test]
    fn test_broadcasts_events() {
        let (tx, rx) = channel();
        let mut broadcast = Broadcast::new(rx);

        let a = broadcast.subscribe();
        let b = broadcast.subscribe();

        let _ = tx.send(123);

        spawn(move || broadcast.start());

        let a_got = a.recv().unwrap();
        let b_got = b.recv().unwrap();

        assert_eq!(a_got, 123);
        assert_eq!(b_got, 123);
    }
}
