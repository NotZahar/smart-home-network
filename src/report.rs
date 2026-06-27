use crate::DeviceResult;

pub trait Report {
    fn report(&self) -> DeviceResult<String>;
}
