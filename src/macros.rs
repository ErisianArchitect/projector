
#[macro_export]
macro_rules! const_try_opt {
    ($value:expr) => {
        if let Some(value) = $value {
            value
        } else {
            return None;
        }
    };
}

pub use crate::const_try_opt;