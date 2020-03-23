/// Amount of time in units of 2^-64 seconds.
pub type Jiffies = u128;

#[derive(Clone, Copy, Debug)]
pub struct SchedulerThreadEntry {
    pub func: fn(),
    pub time: Jiffies, 
}

#[derive(Clone, Copy, Debug)]
pub struct SchedulerThread {
    pub frequency: u128,
    pub scalar: Jiffies,
    pub steps: Jiffies,
    pub next_event_time: Jiffies,
    pub next_event_func: fn(),
    pub entries: Vec<SchedulerThreadEntry>,
}

impl SchedulerThread {
    pub fn new(freq: u128) -> SchedulerThread {
        SchedulerThread {
            frequency: freq,
            // u64::max_value() here is supposed to be the amount of Jiffies one second is.
            scalar: (u64::max_value() as u128 / freq),
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

    pub fn calculate_next_event(&mut self) {
        // iterate through the list
        // find the next event
        // remove events in the past
        let mut next_event: SchedulerThreadEntry;
        let mut next_event_time: u128;
        for i in (0..entries.len()) {
            if entries[i].time <= self.steps {
                // remove entry
                entries.remove(i);
            } else if entries[i].time <= next_event_time {
                next_event_time = entries[i].time;
                next_event = entries[i];
            }
        }

        self.next_event_func = next_event.func;
        self.next_event_time = next_event.time;
    }

    pub fn synchronize(&mut self) {
        if self.steps >= self.next_event_time {
            (self.next_event_func)();// process event
            calculate_next_event();// calculate updated next_event value
        }
    }

    pub fn schedule(&mut self, cycles: u128, func: fn()) {
        // store func + cycles
        entries.push(SchedulerThreadEntry { func: func, time: self.steps + cycles * self.scalar});
        // calculate updated next_event
        calculate_next_event();
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

    // pub fn synchronize(&mut self) {
    //     let mut minimum_val: Jiffies = self.threads[0].steps;

    //     for thread in &self.threads {
    //         if thread.steps < minimum_val {
    //            minimum_val = thread.steps;
    //         }
    //     }

    //     for thread in &mut self.threads {
    //         thread.steps -= minimum_val;
    //     }
    // }
}
