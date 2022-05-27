mod ack_window;
mod common;
mod configuration;
mod control_packet;
mod data_packet;
mod flow;
mod loss_list;
mod multiplexer;
mod packet;
mod queue;
mod seq_number;
mod socket;
mod udt;

pub use socket::UdtSocket;
pub use udt::Udt;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
