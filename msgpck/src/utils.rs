pub fn slice_take<'a, T, const N: usize>(
    s: &mut &'a [T],
) -> Result<&'a [T; N], UnexpectedEofError> {
    if s.len() < N {
        return Err(UnexpectedEofError);
    }

    let head = s[..N].try_into().expect("slice is big enough");
    *s = &s[N..];

    Ok(head)
}

pub struct UnexpectedEofError;
