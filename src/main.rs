extern crate bitflags;

use crate::hardware::*;
use crate::scheduler::*;

pub mod cpu286;
pub mod cpu8086;
pub mod hardware;
pub mod scheduler;

#[allow(dead_code)]

fn main() {
    let mut machine = IbmPc5150Machine::new();
    let mut cpu_thread = SchedulerThread::new(4_772_727);
    let mut scheduler: Scheduler = Scheduler::new();

    scheduler.threads.push(&mut cpu_thread);

    //machine.cpu.tick(&mut machine.hardware);
    loop {
        //let _cycles = cpu_thread.step(4, |_cycles| machine.cpu.tick(&mut machine.hardware));
        //machine.cpu.tick(&mut machine.hardware);
        let _cycles =
            scheduler.threads[0].step(4, |_cycles| machine.cpu.tick(&mut machine.hardware));
        scheduler.synchronize();
    }
}
