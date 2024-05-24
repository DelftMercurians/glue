#[derive(Debug, Copy, Clone)]
pub enum Stamped<T> {
    Have(std::time::Instant, T),
    NothingYet,
}

impl<T> Stamped<T> {
    pub fn make_now(val : T) -> Stamped<T> {
        Self::Have(std::time::Instant::now(), val)
    }

    pub fn update(&mut self, val : T) {
        *self = Self::Have(std::time::Instant::now(), val)
    }

    pub fn time_since(&self) -> Option<std::time::Duration> {
        match self {
            Self::NothingYet => None,
            Self::Have(timestamp, _) => Some(std::time::Instant::now() - *timestamp),
        }
    }
}

#[cfg(test)]
mod stamped_tests {
    use super::*;

    #[test]
    #[allow(unused_assignments)]
    fn stamped() {
        let mut var = Stamped::<u8>::NothingYet;
        var = Stamped::make_now(75);
        std::thread::sleep(std::time::Duration::from_millis(10));
        if let Some(time) = var.time_since() {
            assert!(time > std::time::Duration::from_millis(9));
        } else {
            assert!(false);
        }
    }
}
