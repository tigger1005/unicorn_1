use super::ElfFile;

mod fault_injections;
use fault_injections::*;
pub use fault_injections::{FaultData, FaultType};

use log::debug;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct SimulationData {
    pub address: u64,
    pub size: usize,
    pub count: usize,
    pub fault_type: FaultType,
}

impl SimulationData {
    pub fn set_fault_type(&mut self, fault_type: FaultType) {
        self.fault_type = fault_type;
    }
}

impl fmt::Debug for SimulationData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "address: 0x{:X} size: 0x{:?} count: 0x{:?} fault_type: 0x{:?}",
            self.address, self.size, self.count, self.fault_type
        )
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

    /// Initialize registers and load code. System settings
    /// are set according the given boolean value:
    /// true = The system is setup to run successful
    /// false = The system is setup to run none-successful
    ///
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
    pub fn record_code_trace(&mut self, faults: Vec<SimulationData>) -> Vec<SimulationData> {
        // Initialize and load
        self.init_and_load(false);
        // Deactivate io print
        self.emu.deactivate_printf_function();

        // Set hook with faults and run program
        self.emu.set_trace_hook(faults);
        let _ret = self.emu.run_steps(MAX_INSTRUCTIONS, false);
        self.emu.release_usage_fault_hooks();
        // Convert from hashmap to vector array
        self.convert(self.emu.get_trace())
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
        vec_of_vec_attacks: Vec<Vec<SimulationData>>,
    ) -> Option<Vec<Vec<FaultData>>> {
        self.init_and_load(false);
        // Deactivate io print
        self.emu.deactivate_printf_function();
        //
        let mut fault_data_vec = Vec::new();

        self.emu.init_states(false);
        self.emu.init_register();
        self.emu.context_init();

        vec_of_vec_attacks.iter().for_each(|vec_attacks| {
            self.emu.context_restore();
            // Write code to memory area
            vec_attacks
                .iter()
                .for_each(|attack| self.emu.set_usage_fault_hook(attack.clone()));
            let _ret_val = self.emu.run_steps(MAX_INSTRUCTIONS, false);
            if self.emu.get_state() == RunState::Success {
                fault_data_vec.push(self.emu.get_fault_data());
            }
            self.emu.release_usage_fault_hooks();
        });
        if fault_data_vec.len() != 0 {
            return Some(fault_data_vec);
        }
        None
    }

    pub fn convert(&self, record_map: HashMap<u64, TraceRecord>) -> Vec<SimulationData> {
        let mut list: Vec<SimulationData> = Vec::new();
        record_map.iter().for_each(|record| {
            list.push(SimulationData {
                address: *record.0,
                size: record.1.size,
                count: record.1.count,
                fault_type: FaultType::Uninitialized,
            });
        });

        list
    }
}
