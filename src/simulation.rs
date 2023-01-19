use super::ElfFile;

mod fault_injections;
use fault_injections::*;
pub use fault_injections::{FaultData, FaultType};

use log::debug;
use std::collections::HashMap;

#[derive(Copy, Clone)]
pub struct TraceRecord {
    size: usize,
    count: usize,
}

#[derive(Clone, Copy)]
pub struct SimulationFaultRecord {
    pub address: u64,
    pub size: usize,
    pub count: usize,
    pub fault_type: FaultType,
}

impl SimulationFaultRecord {
    pub fn new(record_map: HashMap<u64, TraceRecord>) -> Vec<SimulationFaultRecord> {
        let mut list: Vec<SimulationFaultRecord> = Vec::new();
        record_map.iter().for_each(|record| {
            list.push(SimulationFaultRecord {
                address: *record.0,
                size: record.1.size,
                count: record.1.count,
                fault_type: FaultType::Uninitialized,
            });
        });

        list
    }
    pub fn set_fault_type(&mut self, fault_type: FaultType) {
        self.fault_type = fault_type;
    }
}

pub struct Simulation<'a> {
    emu: FaultInjections<'a>,
}

impl<'a> Simulation<'a> {
    pub fn new(file_data: &ElfFile) -> Self {
        // Setup emulator
        let mut emu = FaultInjections::new(file_data);
        // Initial setup
        emu.setup_mmio();
        emu.setup_breakpoints();
        Self { emu }
    }

    /// Check if code under investigation is working correct for
    /// positive and negative execution
    ///
    pub fn check_program(&mut self) {
        // Run simulation
        self.run(true);
        assert_eq!(self.emu.get_state(), RunState::Success);

        self.run(false);
        assert_eq!(self.emu.get_state(), RunState::Failed);
    }

    fn init_and_load(&mut self, run_successful: bool) {
        self.emu.init_register();
        // Write code to memory area
        self.emu.load_code();
        // Init state
        self.emu.init_states(run_successful);
    }

    /// Record the program flow till the program ends on positiv or negative program execution
    /// A vector array with the recorded addresses is returned
    ///
    pub fn record_code_trace(
        &mut self,
        external_record: Vec<SimulationFaultRecord>,
    ) -> Vec<SimulationFaultRecord> {
        //
        let mut trace_records_hmap = HashMap::new();
        // Initialize and load
        self.init_and_load(false);
        // Deactivate io print
        self.emu.deactivate_printf_function();

        let (adr, rec) = self.emu.get_cmd_address_record().unwrap();
        trace_records_hmap.insert(adr, rec);

        // Insert nop
        external_record
            .iter()
            .for_each(|record| self.emu.set_fault(*record));

        let mut cycles: usize = 0;

        loop {
            // Do one step in code
            if self.emu.run_steps(1, false) != Ok(()) {
                break;
            }
            // Handle cycles (cpu command steps)
            cycles += 1;
            if cycles > MAX_INSTRUCTIONS {
                break;
            }
            // Write next execution address to hash map
            if let Some((adr, rec)) = self.emu.get_cmd_address_record() {
                trace_records_hmap
                    .entry(adr)
                    .and_modify(|record| record.count += 1)
                    .or_insert(rec);
            } else {
                break;
            }
            // If failed marker is written -> stop recording
            if self.emu.get_state() == RunState::Failed {
                break;
            }
        }
        // Convert hash map to vector array
        SimulationFaultRecord::new(trace_records_hmap)
    }

    fn run(&mut self, run_successful: bool) {
        let ret_info = self.run_till(run_successful, MAX_INSTRUCTIONS);

        if ret_info == Ok(()) {
            debug!("Program stopped successful");
        } else {
            debug!("Program stopped with {:?}", ret_info);
        }
        //print_register_and_data(emu);
    }

    fn run_till(&mut self, run_successful: bool, steps: usize) -> Result<(), uc_error> {
        self.init_and_load(run_successful);
        // Start execution
        debug!("Run : {} Steps", steps);
        self.emu.run_steps(steps, false)
    }

    /// Execute loaded code with the given faults injected bevor code execution
    /// If code finishes with successful state, a vector array will be returned with the
    /// injected faults
    ///
    pub fn run_with_faults(
        &mut self,
        external_record: Vec<SimulationFaultRecord>,
    ) -> Option<Vec<FaultData>> {
        self.init_and_load(false);
        // Deactivate io print
        self.emu.deactivate_printf_function();
        // Set nop
        external_record
            .iter()
            .for_each(|record| self.emu.set_fault(*record));
        // Run
        let _ret_val = self.emu.run_steps(MAX_INSTRUCTIONS, false);
        if self.emu.get_state() == RunState::Success {
            return Some(self.emu.get_fault_data().clone());
        }
        None
    }
}
