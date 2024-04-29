#[derive(Debug, Clone, Copy)]
pub enum CpuState {
    Fetch,
    FetchPrefix,
    Execute { machine_cycle: u8, temp_reg: u16, is_prefix: bool },   //Machine cycle will help us know which step if the instruction were on. And temp will help persist values 
    Halt,
}

#[allow(unused)]
pub enum Status {
    Completed,
    Running,
    Error,
}