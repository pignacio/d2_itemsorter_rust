macro_rules! bitsy_read {
    ($reader:ident $(, $dest:ident $(: $type:ty)?)+ $(,)?) => {
        $(let $dest $(: $type)? = $reader.read().prepend_path(stringify!($dest))?;)+
    };
}

pub(crate) use bitsy_read;

macro_rules! bitsy_write {
    ($writer:ident $(, &$value:ident)+ $(,)?) => {
        $( $writer.write(&$value).prepend_path(stringify!($value))?; )+
    };
    ($writer:ident $(, &$self:ident.$value:ident)+ $(,)?) => {
        $(
        $writer
            .write(&$self.$value)
            .prepend_path(stringify!($value))?;
        )+
    };
}
pub(crate) use bitsy_write;
