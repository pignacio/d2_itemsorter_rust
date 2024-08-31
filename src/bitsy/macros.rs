macro_rules! bitsy_read {
    ($reader:ident $(, $dest:ident $(: $type:ty)?)+ $(,)?) => {
        $(let $dest $(: $type)? = $crate::bitsy::error::BitsyErrorExt::prepend_path($reader.read(), stringify!($dest))?;)+
    };
}

pub(crate) use bitsy_read;

macro_rules! bitsy_write {
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
