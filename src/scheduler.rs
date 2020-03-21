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

    pub fn step(&mut self, cycles: u128) {
        self.steps += self.scalar * cycles;
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

    pub fn synchronize(&mut self) -> Jiffies {
        let mut minimum_val: Jiffies = self.threads[0].steps;

        for thread in &self.threads {
            if thread.steps < minimum_val {
                minimum_val = thread.steps;
            }
        }

        for thread in &mut self.threads {
            thread.steps -= minimum_val;
        }

        let threshold: Jiffies  = 10;
        let mut min_steps: Jiffies = 0;
        let mut max_steps: Jiffies = 0;
        for thread in &self.threads {
            if thread.steps > max_steps {
                max_steps = thread.steps;
            }
        }
        loop {
            if max_steps >= threshold {
                for thread in &mut self.threads {
                    thread.step(threshold);
                    min_steps += threshold;
                }
                max_steps -= threshold;
            }
            else {
                break;
            }
        }

        min_steps
    }
}