pub type HandlerFunc = unsafe extern "C" fn();

#[repr(C)]
pub struct ScratchRegisters {
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
}

impl ScratchRegisters {
    pub fn dump(&self) {
        log::debug!("rax: {:#x}", self.rax);
        log::debug!("rcx: {:#x}", self.rcx);
        log::debug!("rdx: {:#x}", self.rdx);
        log::debug!("rdi: {:#x}", self.rdi);
        log::debug!("rsi: {:#x}", self.rsi);
        log::debug!("r8: {:#x}", self.r8);
        log::debug!("r9: {:#x}", self.r9);
        log::debug!("r10: {:#x}", self.r10);
        log::debug!("r11: {:#x}", self.r11);
    }
}

#[repr(C)]
pub struct PreservedRegisters {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
}

impl PreservedRegisters {
    pub fn dump(&self) {
        log::debug!("rbx: {:#x}", self.rbx);
        log::debug!("rbp: {:#x}", self.rbp);
        log::debug!("r12: {:#x}", self.r12);
        log::debug!("r13: {:#x}", self.r13);
        log::debug!("r14: {:#x}", self.r14);
        log::debug!("r15: {:#x}", self.r15);
    }
}

#[repr(C)]
pub struct IretRegisters {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl IretRegisters {
    pub fn dump(&self) {
        log::debug!("rip: {:#x}", self.rip);
        log::debug!("cs: {:#x}", self.cs);
        log::debug!("rflags: {:#x}", self.rflags);
        log::debug!("rsp: {:#x}", self.rsp);
        log::debug!("ss: {:#x}", self.ss);
    }
}

#[repr(C)]
pub struct InterruptStackFrame {
    pub scratch: ScratchRegisters,
    pub preserved: PreservedRegisters,
    pub iret: IretRegisters,
}

impl InterruptStackFrame {
    pub fn dump(&self) {
        self.scratch.dump();
        self.preserved.dump();
        self.iret.dump();
    }
}

#[macro_export]
macro_rules! push_scratch {
    () => {
        "
            push rcx
            push rdx
            push rdi
            push rsi
            push r8
            push r9
            push r10
            push r11
        "
    };
}

#[macro_export]
macro_rules! pop_scratch {
    () => {
        "
            pop r11
            pop r10
            pop r9
            pop r8
            pop rsi
            pop rdi
            pop rdx
            pop rcx
        "
    };
}

#[macro_export]
macro_rules! push_preserved {
    () => {
        "
            push rbx
            push rbp
            push r12
            push r13
            push r14
            push r15
        "
    };
}

#[macro_export]
macro_rules! pop_preserved {
    () => {
        "
            pop r15
            pop r14
            pop r13
            pop r12
            pop rbp
            pop rbx
        "
    };
}

#[macro_export]
macro_rules! interrupt_stack {
    ($name:ident, |$stack:ident| $code:block) => {
        #[unsafe(naked)]
        pub unsafe extern "C" fn $name() {
            extern "C" fn inner($stack: &mut $crate::arch::interrupts::handler::InterruptStackFrame) {
                $code
            }

            core::arch::naked_asm!(concat!(
                "cld;",
                "push rax\n",
                $crate::push_scratch!(),
                $crate::push_preserved!(),
                "mov rdi, rsp;",
                "call {inner}",
                $crate::pop_preserved!(),
                $crate::pop_scratch!(),
                "iretq\n"
            ), inner = sym inner,);
        }
    };
}

#[macro_export]
macro_rules! interrupt_error {
    ($name:ident, |$stack:ident, $error_code:ident| $code:block) => {
        #[unsafe(naked)]
        pub unsafe extern "C" fn $name() {
            extern "C" fn inner($stack: &mut $crate::arch::interrupts::handler::InterruptStackFrame, $error_code: u64) {
                $code
            }

            core::arch::naked_asm!(concat!(
                "cld;",
                $crate::push_scratch!(),
                $crate::push_preserved!(),
                "mov rsi, [rsp + {rax_offset}];",
                "mov [rsp + {rax_offset}], rax;",
                "mov rdi, rsp;"
                "call {inner}",
                $crate::pop_preserved!(),
                $crate::pop_scratch!(),
                "iretq\n"
            ), inner = sym inner,
                rax_offset = const(::core::mem::size_of::<$crate::arch::x86_64::interrupts::handler::PreservedRegisters>() + ::core::mem::size_of::<$crate::arch::x86_64::interrupts::handler::ScratchRegisters>() - 8),
            );
        }
    };
}
