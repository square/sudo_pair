use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub struct ParseListError();

pub trait FromSudoOption: Sized {
    type Err;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err>;
}

impl FromSudoOption for bool {
    type Err = ::std::str::ParseBoolError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for i8 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for u8 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for i16 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}


impl FromSudoOption for u16 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for i32 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for u32 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for i64 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for u64 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for String {
    type Err = ::std::string::ParseError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for PathBuf {
    type Err = ::std::string::ParseError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl<T> FromSudoOption for Vec<T>
where
    T: FromSudoOption + FromSudoOptionList,
{
    type Err = ParseListError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let      list = <T as FromSudoOptionList>::from_sudo_option_list(s);
        let mut items = Self::with_capacity(list.len());

        for element in list {
            let item = FromSudoOption::from_sudo_option(element)
                .map_err(|_| ParseListError())?;

            items.push(item);
        }

        Ok(items)
    }
}

pub trait FromSudoOptionList: Sized {
    const SEPARATOR: char = ',';

    fn from_sudo_option_list(s: &str) -> Vec<&str> {
        s.split(|b| b == Self::SEPARATOR).collect()
    }
}

impl FromSudoOptionList for i8 {}
impl FromSudoOptionList for u8 {}
impl FromSudoOptionList for i16 {}
impl FromSudoOptionList for u16 {}
impl FromSudoOptionList for i32 {}
impl FromSudoOptionList for u32 {}
impl FromSudoOptionList for i64 {}
impl FromSudoOptionList for u64 {}
