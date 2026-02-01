use std::marker::PhantomData;
use crate::mvc::model::Model;
pub struct QueryBuilder<M: Model> {
    select: SelectQuery,
    _marker: PhantomData<M>,
}

impl<M: Model> QueryBuilder<M> {
    pub fn all() -> Self {
        println!("qb all called 2");
        QueryBuilder {
            select: SelectQuery {
                table: M::table(),
                columns: M::columns(),
            },
            _marker: PhantomData,
        }
    }

    // temp just for watch what we did
    pub fn debug(self) -> SelectQuery {
        self.select
    }
}


#[derive(Debug)]
pub struct SelectQuery {
    pub table: &'static str,
    pub columns: &'static [&'static str],
}

pub trait QueryDsl: Model {
    fn all() -> QueryBuilder<Self>;
}

impl<T: Model> QueryDsl for T {
    fn all() -> QueryBuilder<Self> {
        println!("qb all called 1");
        QueryBuilder::all()
    }
}