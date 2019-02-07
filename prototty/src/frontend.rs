pub trait Frontend: private::Sealed {}

pub struct Glutin;
impl Frontend for Glutin {}

pub struct Unix;
impl Frontend for Wasm {}

pub struct Wasm;
impl Frontend for Unix {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Glutin {}
    impl Sealed for super::Unix {}
    impl Sealed for super::Wasm {}
}
