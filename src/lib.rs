use std::sync::OnceLock;

pub mod app;
pub mod types;
pub mod worker;

pub fn default_time_format() -> &'static [time::format_description::FormatItem<'static>] {
    static MEM: OnceLock<&[time::format_description::FormatItem<'static>]> = OnceLock::new();
    MEM.get_or_init(|| {
        time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second] [offset_hour sign:mandatory]"
        )
    })
}

/// This macro is for tracing error and returning Result if there are some
/// meaningful Ok() case, and returning () if there are no meaningful result.
/// It is useful to simply trace error message on fallible operations which doesn't
/// return anything in the Ok() branch.
#[macro_export]
macro_rules! trace_err {
    ($exp:expr) => {
        match $exp {
            Ok(v) => Ok(v),
            Err(e) => {
                println!("{e}");
                Err(e)
            }
        }
    };
    ($exp:expr, ()) => {
        match $exp {
            Ok(()) => (),
            Err(e) => {
                println!("{e}");
                ()
            }
        }
    };
}
