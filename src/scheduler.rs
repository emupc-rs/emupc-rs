/// Amount of time in units of 2^-128 seconds.
pub type Jiffies = u128;

#[derive(Clone, Copy, Debug)]
pub struct SchedulerThread {
    pub frequency: u128,
    pub scalar: Jiffies,
    pub steps: Jiffies,
}

impl SchedulerThread {
    pub fn new(freq: u128) -> SchedulerThread {
        SchedulerThread {
            frequency: freq,
            // u128::MAX here is supposed to be the amount of Jiffies one second is.
            scalar: (u128::max_value() / freq),
            steps: 0,
        }
    }

    pub fn step<F>(&mut self, cycles: u128, mut func: F) -> u128
    where
        F: FnMut(u128) -> u128,
    {
        self.steps += self.scalar * cycles;
        func(cycles)
    }
}

#[derive(Debug)]
pub struct Scheduler {
    pub threads: Vec<SchedulerThread>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            threads: Vec::new(),
        }
    }

    pub fn synchronize(&mut self) {
        let mut minimum_val: Jiffies = self.threads[0].steps;

        for thread in &self.threads {
            if thread.steps < minimum_val {
                minimum_val = thread.steps;
            }
        }

        for thread in &mut self.threads {
            thread.steps -= minimum_val;
        }
    }
}
