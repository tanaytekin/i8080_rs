pub struct I8080 {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    flags: u8,
    cycles: usize,
    memory: Box<[u8]>,
}

impl I8080 {
    pub fn new(memory_size: usize) -> Self {
        Self {
            pc: 0,
            sp: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            flags: 0b00000010, // always: bit-1 = 1, bit-5 = 0
            cycles: 0,
            memory: vec![0; memory_size].into_boxed_slice(),
        }
    }

    pub fn cycle(&mut self) {
        if self.cycles > 0 {
            self.cycles -= 1;
            return;
        }

        let opcode = self.next_u8();
        let cycles = match opcode {
            _ => {eprintln!("Invalid opcode: {opcode}"); 0}
        };

        self.cycles += cycles;
    }

    fn read_u8(&self, location: u16) -> u8 {
        self.memory[location as usize]
    }

    fn next_u8(&mut self) -> u8 {
        let value = self.read_u8(self.pc);
        self.pc += 1;
        value
    }
}
