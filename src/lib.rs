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
                                                                        
            0xC1 => {self.pop(RegisterPair::B); 10},                    // POP B
            0xD1 => {self.pop(RegisterPair::D); 10},                    // POP D
            0xE1 => {self.pop(RegisterPair::H); 10},                    // POP H
            0xF1 => {self.pop(RegisterPair::PSW); 10},                  // POP PSW
            
            0xC5 => {self.push(RegisterPair::B); 11},                   // PUSH B
            0xD5 => {self.push(RegisterPair::D); 11},                   // PUSH D
            0xE5 => {self.push(RegisterPair::H); 11},                   // PUSH H
            0xF5 => {self.push(RegisterPair::PSW); 11},                 // PUSH PSW
            
            0x22 => {self.shld(); 16},                                  // SHLD a16
            0x2A => {self.lhld(); 16},                                  // LHLD a16
            0xE3 => {self.xthl(); 18},                                  // XTHL
            0xF9 => {self.sphl(); 5},                                   // SPHL
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
    
    fn write_u8(&mut self, location: u16, value: u8) {
        self.memory[location as usize] = value;
    }
    
    fn write_u16(&mut self, location: u16, value: u16) {
        let value = value.to_le_bytes();
        self.memory[location as usize] = value[0];
        self.memory[(location + 1) as usize] = value[1];
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
 
    fn register_pair_to_refs(&self, pair: RegisterPair) -> (&u8, &u8) {
        match pair {
            RegisterPair::B => (&self.b, &self.c),
            RegisterPair::D => (&self.d, &self.e),
            RegisterPair::H => (&self.h, &self.l),
            RegisterPair::PSW => (&self.a, &self.flags),
        }
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

    fn get_register_pair(&mut self, pair: RegisterPair) -> u16 {
        let (high, low) = self.register_pair_to_refs(pair);
        ((*high as u16) << 8) | (*low as u16)
    }

    fn lxi(&mut self, pair: RegisterPair) {
        let value = self.next_u16();
        self.set_register_pair(pair, value);
    }
    
    fn lxi_sp(&mut self) {
        self.sp = self.next_u16();
    }

    fn pop(&mut self, pair: RegisterPair) {
        eprintln!("{}",self.sp);
        let value = self.read_u16(self.sp);
        self.set_register_pair(pair, value);
        self.sp += 2;
    }

    fn push(&mut self, pair: RegisterPair) {
        let value = self.get_register_pair(pair);
        self.sp -= 2;
        self.write_u16(self.sp, value);
    }

    fn shld(&mut self) {
        let location = self.next_u16();
        let value = self.get_register_pair(RegisterPair::H);
        self.write_u16(location, value);
    }

    fn lhld(&mut self) {
        let location = self.next_u16();
        let value = self.read_u16(location);
        self.set_register_pair(RegisterPair::H, value);
    }

    fn xthl(&mut self) {
        let register = self.get_register_pair(RegisterPair::H);
        let stack = self.read_u16(self.sp);
        self.set_register_pair(RegisterPair::H, stack);
        self.write_u16(self.sp, register);
    }

    fn sphl(&mut self) {
        self.sp = self.get_register_pair(RegisterPair::H);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTS_DEFAULT_MEMORY_SIZE: usize = 1024;
    const TESTS_DEFAULT_SP: u16 = TESTS_DEFAULT_MEMORY_SIZE as u16 / 2;
    macro_rules! i8080 {
        () => {
            {
                let mut i8080 = I8080::new(TESTS_DEFAULT_MEMORY_SIZE);
                i8080.sp = TESTS_DEFAULT_SP;
                i8080
            }
        };
        ( $( $x:expr ), * ) => {
            {
                let mut i8080 = I8080::new(TESTS_DEFAULT_MEMORY_SIZE);
                i8080.sp = TESTS_DEFAULT_SP;
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
        #[test]
        fn pop() {
            let mut i8080 = i8080!();
            i8080.write_u8(i8080.sp, 0x3D);
            i8080.write_u8(i8080.sp + 1, 0x93);
            i8080.pop(RegisterPair::H);
            assert_eq!(i8080.h, 0x93);
            assert_eq!(i8080.l, 0x3D);
            assert_eq!(i8080.sp, TESTS_DEFAULT_SP + 2);
        }

        #[test]
        fn push() {
            let mut i8080 = i8080!();
            i8080.d = 0x8F;
            i8080.e = 0x9D;
            i8080.push(RegisterPair::D);
            assert_eq!(i8080.d, 0x8F);
            assert_eq!(i8080.e, 0x9D);
            assert_eq!(i8080.sp, TESTS_DEFAULT_SP - 2);
            assert_eq!(i8080.read_u8(i8080.sp), 0x9D);
            assert_eq!(i8080.read_u8(i8080.sp + 1), 0x8F);
        }
        #[test]
        fn shld() {
            let mut i8080 = i8080![0xA, 0x1];
            i8080.h = 0xAE;
            i8080.l = 0x29;
            i8080.shld();
            assert_eq!(i8080.read_u8(0x10A), 0x29);
            assert_eq!(i8080.read_u8(0x10B), 0xAE);
        }
        #[test]
        fn lhld() {
            let mut i8080 = i8080![0x5B, 0x2];
            i8080.write_u8(0x25B, 0xFF);
            i8080.write_u8(0x25C, 0x03);
            i8080.lhld();
            assert_eq!(i8080.l, 0xFF);
            assert_eq!(i8080.h, 0x03);
        }
        #[test]
        fn xthl() {
            let mut i8080 = i8080!();
            i8080.write_u8(i8080.sp, 0xF0);
            i8080.write_u8(i8080.sp + 1, 0x0D);
            i8080.h = 0x0B;
            i8080.l = 0x3C;
            i8080.xthl();
            assert_eq!(i8080.read_u8(i8080.sp), 0x3C);
            assert_eq!(i8080.read_u8(i8080.sp + 1), 0x0B);
            assert_eq!(i8080.h, 0x0D);
            assert_eq!(i8080.l, 0xF0);
        }
        #[test]
        fn sphl() {
            let mut i8080 = i8080!();
            i8080.h = 0x50;
            i8080.l = 0x6C;
            i8080.sphl();
            assert_eq!(i8080.sp, 0x506C);
        }
    }
}
