extern crate bitflags;

use crate::cpu8086::*;
use crate::hardware::*;

pub mod cpu286;
pub mod cpu8086;
pub mod hardware;

#[allow(dead_code)]

fn main() {
    let mut machine = IbmPc5150Machine::new();
    //let mut cpu_thread = SchedulerThread::new(4_772_727);
    //let mut pit_thread = SchedulerThread::new(1_193_182);
    //let mut scheduler: Scheduler<IbmPc5150Machine> = Scheduler::new();

    //scheduler.threads.push(&mut cpu_thread);
    //scheduler.threads.push(&mut pit_thread);
    //let pit_func = IbmPc5150Machine::tick;
    //let cpu_func = Cpu8086::tick;
    //scheduler.threads[1].schedule(4, pit_func, &mut machine);
    //scheduler.threads[0].schedule(1, cpu_func, &mut machine.cpu);

    loop {
        let cycles: usize = machine.cpu.tick(&mut machine.hardware);
        machine.tick(cycles);
    }
}
