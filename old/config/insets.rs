use std::{fmt::Display, str::FromStr};

use iced::runtime::platform_specific::wayland::layer_surface::IcedMargin;
use serde::Deserialize;
use toml::Value;

#[derive(Debug, Default)]
pub struct Insets<T> {
    t: T,
    b: T,
    l: T,
    r: T,
}

impl<T: Copy> Insets<T> {
    pub fn all(v: T) -> Self {
        Insets {
            t: v,
            b: v,
            l: v,
            r: v,
        }
    }
}

impl From<&Insets<i32>> for IcedMargin {
    fn from(p: &Insets<i32>) -> Self {
        IcedMargin {
            top: p.t,
            right: p.r,
            bottom: p.b,
            left: p.l,
        }
    }
}

impl<'de, T> Deserialize<'de> for Insets<T>
where
    T: FromStr + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.trim().split_whitespace().collect();

        let parse = |v: &str| {
            v.parse().map_err(|_| {
                serde::de::Error::invalid_type(serde::de::Unexpected::Str(v), &"a number")
            })
        };

        if let [t, r, b, l] = parts[..] {
            return Ok(Insets {
                t: parse(t)?,
                b: parse(b)?,
                l: parse(l)?,
                r: parse(r)?,
            });
        } else if let [v, h] = parts[..] {
            let v = parse(v)?;
            let h = parse(h)?;
            return Ok(Insets {
                t: v,
                b: v,
                l: h,
                r: h,
            });
        } else if let [all] = parts[..] {
            return Ok(Insets::all(parse(all)?));
        }

        Err(serde::de::Error::invalid_length(
            4,
            &"expected 1, 2 or 4 arguments",
        ))
    }
}

impl<T> ToString for Insets<T>
where
    T: Display + PartialEq,
{
    fn to_string(&self) -> String {
        if self.t == self.b && self.t == self.l && self.t == self.r {
            format!("{}", self.t)
        } else if self.t == self.b && self.l == self.r {
            format!("{} {}", self.t, self.l)
        } else {
            format!("{} {} {} {}", self.t, self.r, self.b, self.l)
        }
    }
}

impl<T> From<Insets<T>> for Value
where
    T: Display + PartialEq,
{
    fn from(value: Insets<T>) -> Self {
        Value::String(value.to_string())
    }
}
