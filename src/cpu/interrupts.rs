

pub mod interrupts {
    use crate::cpu::CPU::CPU;
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    #[derive(Debug, EnumIter)]
    pub enum InterruptType {
     VBlank = 1,
     LCD_STAT = 2,
     Timer = 4,
     Serial = 8,
     Joypad = 16,
    }

    impl CPU<'_> {
        pub(crate) fn check_interrupts(&mut self) {
            let master_interrupts_enabled = self.MMU.interrupt_master_enabled != 0;
            let interrupts_enabled = self.MMU.interrupt_enabled != 0;
            let interrupts_flags_enabled = (self.MMU.interrupt_flag & 0x1F) != 0;

            if master_interrupts_enabled && interrupts_enabled && interrupts_flags_enabled {

                for interrupt_type in InterruptType::iter() {
                    let interrupt_bit = interrupt_type as u8;
                    if (self.MMU.interrupt_enabled & interrupt_bit != 0) && (self.MMU.interrupt_flag & interrupt_bit != 0) {
                        self.MMU.interrupt_flag = self.MMU.interrupt_flag & !interrupt_bit;
                        self.MMU.interrupt_master_enabled = 0;
                        self.handle_interrupt(interrupt_type)
                    }
                }

            }
        }

        pub(crate) fn handle_interrupt(&mut self, interrupt_type: InterruptType) {
            match interrupt_type {
                InterruptType::VBlank => {

                }
                InterruptType::LCD_STAT => {

                }
                InterruptType::Timer => {

                }
                InterruptType::Serial => {

                }
                InterruptType::Joypad => {

                }
            }
        }
    }
}