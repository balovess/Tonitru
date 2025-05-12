// Tonitru library entry point
// Core modules will be defined here

pub mod codec;
pub mod internal;
pub mod compress; // Declare the compress module

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}