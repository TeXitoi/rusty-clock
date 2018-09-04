use heapless::{consts::*, Vec};
use ui::Msg;

pub struct MsgQueue {
    q: Vec<Msg, U16>,
}
impl MsgQueue {
    pub fn new() -> Self {
        Self { q: Vec::new() }
    }
    pub fn push(&mut self, msg: Msg) {
        if let Err(_) = self.q.push(msg) {
            panic!("msg queue full");
        }
        ::rtfm::set_pending(::hal::stm32f103xx::Interrupt::EXTI2);
    }
    pub fn get(&mut self) -> Vec<Msg, U16> {
        ::core::mem::replace(&mut self.q, Vec::new())
    }
}
