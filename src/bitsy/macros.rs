macro_rules! bitsy_read {
    ($reader:ident $(, $dest:ident $(: $type:ty)?)+ $(,)?) => {
        $(let $dest $(: $type)? = $crate::bitsy::error::BitsyErrorExt::prepend_path($reader.read(), stringify!($dest))?;)+
    };
}

pub(crate) use bitsy_read;

macro_rules! bitsy_write {
    ($writer:ident $(, $value:ident)+ $(,)?) => {
        $( $crate::bitsy::error::BitsyErrorExt::prepend_path($writer.write($value), stringify!($value))?; )+
    };
    ($writer:ident $(, &$value:ident)+ $(,)?) => {
        $( $crate::bitsy::error::BitsyErrorExt::prepend_path($writer.write(&$value), stringify!($value))?; )+
    };
    ($writer:ident $(, &$self:ident.$value:ident)+ $(,)?) => {
        $(
        $crate::bitsy::error::BitsyErrorExt::prepend_path(
          $writer.write(&$self.$value),
          stringify!($value))?;
        )+
    };
}
pub(crate) use bitsy_write;

macro_rules! bitsy_cond_read {
    ($reader:ident, $cond:expr $(, $dest:ident $(: $type:ty)?)+ $(,)?) => {
        $(
        let $dest $(: $type)? = if $cond {
            Some($crate::bitsy::error::BitsyErrorExt::prepend_path($reader.read(), stringify!($dest))?)
        } else {
            None
        };
        )+
    };
}
pub(crate) use bitsy_cond_read;
