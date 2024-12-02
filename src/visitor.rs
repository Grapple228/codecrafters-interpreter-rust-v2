
pub trait Visitor<T> {
    fn visit(&self, acceptor: impl Acceptor<T, Self>) -> T
    where
        Self: Sized;
}

pub trait Acceptor<T, V>
where
    V: Visitor<T>,
{
    fn accept(&self, visitor: V) -> T;
}
