pub struct State {
    count: u32,
}

impl State {
    pub fn new() -> State {
        State{ count:0 }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }
}
