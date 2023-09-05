pub enum CpuState {
    Fetch,
    FetchPrefix,
    Execute { machine_cycle: u8, temp_reg: u16 },   //Machine cycle will help us know which step if the instruction were on. And temp will help persist values 
    InterruptHandle,
}

pub enum ExecuteStatus {
    Completed,
    Running,
    Error,
}
