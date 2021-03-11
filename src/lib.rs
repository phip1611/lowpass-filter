// #![no_std]
// extern crate alloc;

pub use audio_visualizer::{
    Channels,
    ChannelInterleavement
};

pub mod simple;
mod test;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
