/// Amount of time in units of 2^-64 seconds.
pub type Jiffies = u128;

#[derive(Clone, Debug)]
pub struct SchedulerThreadEntry {
    pub func: fn(),
    pub time: Jiffies,
}

#[derive(Clone, Debug)]
pub struct SchedulerThread {
    pub frequency: u128,
    pub scalar: Jiffies,
    pub steps: Jiffies,
    pub next_event_time: Jiffies,
    pub entries: Vec<SchedulerThreadEntry>,
}

impl SchedulerThread {
    pub fn new(freq: u128) -> SchedulerThread {
        SchedulerThread {
            frequency: freq,
            // u64::max_value() here is supposed to be the amount of Jiffies one second is.
            scalar: (u64::max_value() as u128 / freq),
            steps: 0,
            next_event_time: 0,
            entries: Vec::new(),
        }
    }

    pub fn step<F>(&mut self, cycles: u128, mut func: F) -> u128
    where
        F: FnMut(u128) -> u128,
    {
        self.steps += self.scalar * cycles;
        func(cycles)
    }

    pub fn calculate_next_event(&mut self) {
        // iterate through the list
        // find the next event
        // remove events in the past
        let mut next_event_time: u128 = 0;
        for i in 0..self.entries.len() {
            if self.entries[i].time <= self.steps {
                // run the callback
                (self.entries[i].func)();
            // remove entry
            // self.entries.remove(i);
            } else if self.entries[i].time <= next_event_time {
                next_event_time = self.entries[i].time;
            }
        }

        self.next_event_time = next_event_time;
    }

    pub fn synchronize(&mut self) {
        self.steps += self.scalar;
        if self.steps >= self.next_event_time {
            self.calculate_next_event(); // calculate updated next_event value
        }
    }

    pub fn schedule(&mut self, cycles: u128, func: fn()) {
        // store func + cycles
        self.entries.push(SchedulerThreadEntry {
            func: func,
            time: self.steps + cycles * self.scalar,
        });
        // calculate updated next_event
        self.calculate_next_event();
    }
}

#[derive(Debug)]
pub struct Scheduler<'a> {
    pub threads: Vec<&'a mut SchedulerThread>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self{
        Scheduler {
            threads: Vec::new(),
        }
    }

    pub fn synchronize(&mut self) {
        let minimum_val: Jiffies = self
            .threads
            .iter()
            .min_by_key(|x| -> Jiffies { x.steps })
            .unwrap()
            .steps;

        for thread in &mut self.threads {
            thread.steps -= minimum_val;
            thread.synchronize();
        }
    }
}

impl<'a> Default for Scheduler<'a> {
    fn default() -> Self {
        Self::new()
    }
}