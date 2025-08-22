use crate::helpers::accept_option::AcceptOption;

pub fn overwrite_none<T, V>(left: &mut T, right: T)
where
    T: AcceptOption<V>,
{
    let opt_left = left.as_opt();
    let opt_right = right.into_opt();
    if opt_left.is_none() && opt_right.is_some() {
        *left = AcceptOption::from_opt(opt_right);
    }
}
