pub trait Frontend: private::Sealed + Copy {
    fn can_quit() -> bool;
    fn can_save_from_menu() -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct Glutin;
impl Frontend for Glutin {
    fn can_quit() -> bool {
        true
    }
    fn can_save_from_menu() -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Unix;
impl Frontend for Unix {
    fn can_quit() -> bool {
        true
    }
    fn can_save_from_menu() -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Wasm;
impl Frontend for Wasm {
    fn can_quit() -> bool {
        false
    }
    fn can_save_from_menu() -> bool {
        false
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Glutin {}
    impl Sealed for super::Unix {}
    impl Sealed for super::Wasm {}
}
