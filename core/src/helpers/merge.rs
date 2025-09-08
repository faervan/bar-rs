use crate::helpers::accept_option::AcceptOption;

pub fn overwrite_if_some<T, V>(left: &mut T, right: T)
where
    T: AcceptOption<V>,
{
    let opt_right = right.into_opt();
    if opt_right.is_some() {
        *left = AcceptOption::from_opt(opt_right);
    }
}

pub fn overwrite_or_append<T, V, I>(left: &mut T, right: T)
where
    T: AcceptOption<V>,
    V: Extend<I> + IntoIterator<Item = I>,
{
    let opt_left = left.as_opt_mut();
    let opt_right = right.into_opt();
    if let Some(right) = opt_right {
        match opt_left {
            Some(left) => left.extend(right),
            None => *left = AcceptOption::from_opt(Some(right)),
        }
    }
}
