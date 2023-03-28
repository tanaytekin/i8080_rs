enum RegisterPair {
    B,
    D,
    H,
    PSW,
}

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
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 => 4, // NOP
            0x01 => {self.lxi(RegisterPair::B); 10},                    // LXI B,d16
            0x11 => {self.lxi(RegisterPair::D); 10},                    // LXI D,d16
            0x21 => {self.lxi(RegisterPair::H); 10},                    // LXI D,d16
            0x31 => {self.lxi_sp(); 10},                                // LXI SP,d16
            _ => {eprintln!("Invalid opcode: {opcode}"); 0}
        };

        self.cycles += cycles;
    }

    fn read_u8(&self, location: u16) -> u8 {
        self.memory[location as usize]
    }
    
    fn read_u16(&self, location: u16) -> u16 {
        ((self.memory[(location + 1) as usize] as u16) << 8) | self.memory[location as usize] as u16
    }

    fn next_u8(&mut self) -> u8 {
        let value = self.read_u8(self.pc);
        self.pc += 1;
        value
    }

    fn next_u16(&mut self) -> u16 {
        let value = self.read_u16(self.pc);
        self.pc += 2;
        value
    }

    fn register_pair_to_mut_refs(&mut self, pair: RegisterPair) -> (&mut u8, &mut u8) {
        match pair {
            RegisterPair::B => (&mut self.b, &mut self.c),
            RegisterPair::D => (&mut self.d, &mut self.e),
            RegisterPair::H => (&mut self.h, &mut self.l),
            RegisterPair::PSW => (&mut self.a, &mut self.flags),
        }
    }

    fn set_register_pair(&mut self, pair: RegisterPair, value: u16) {
        let value = value.to_le_bytes();
        let (high, low) = self.register_pair_to_mut_refs(pair);
        *low = value[0];
        *high = value[1];
    }

    fn lxi(&mut self, pair: RegisterPair) {
        let value = self.next_u16();
        self.set_register_pair(pair, value);
    }
    
    fn lxi_sp(&mut self) {
        self.sp = self.next_u16();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTS_DEFAULT_MEMORY_SIZE: usize = 1024;
    macro_rules! i8080 {
        () => (I8080::new(TESTS_DEFAULT_MEMORY_SIZE));
        ( $( $x:expr ), * ) => {
            {
                let mut i8080 = I8080::new(TESTS_DEFAULT_MEMORY_SIZE);
                let mut index = 0;
                $(
                    #[allow(unused_assignments)]
                    {
                        i8080.memory[index] = $x;
                        index += 1;
                    }
                 )*
                i8080
            }
        };
    }

    #[cfg(test)]
    mod util_tests {
        use super::*;
        #[test]
        fn set_register_pair() {
            let mut i8080 = i8080!();
            i8080.set_register_pair(RegisterPair::H, 0x1234);
            assert_eq!(i8080.h, 0x12);
            assert_eq!(i8080.l, 0x34);
        }
    }

    #[cfg(test)]
    mod opcode_tests {
        use super::*;
        #[test]
        fn lxi() {
            let mut i8080 = i8080![0x3, 0x1];
            i8080.lxi(RegisterPair::H);
            assert_eq!(i8080.h, 0x1);
            assert_eq!(i8080.l, 0x3);
        }
        #[test]
        fn lxi_sp() {
            let mut i8080 = i8080![0xBC, 0x3A];
            i8080.lxi_sp();
            assert_eq!(i8080.sp, 0x3ABC);
        }
    }
}
