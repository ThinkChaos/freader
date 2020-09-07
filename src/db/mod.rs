use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql};
use diesel::sql_types::Integer;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;

mod executor;
mod helper;
pub mod models;
mod schema;

pub use executor::Executor;
pub use helper::{Error, Helper};


#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[derive(Hash, Eq, PartialEq)] // for diesel
#[derive(AsExpression, FromSqlRow)]
#[sql_type = "Integer"]
pub struct Id(i32);

impl Id {
    pub fn from_raw(value: i32) -> Self {
        Self(value)
    }

    pub fn inner(self) -> i32 {
        self.0
    }
}

impl FromStr for Id {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Id)
    }
}

impl<DB> FromSql<Integer, DB> for Id
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(value: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let id = <i32 as FromSql<Integer, DB>>::from_sql(value)?;

        Ok(Id(id))
    }
}

impl<DB> ToSql<Integer, DB> for Id
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
        self.0.to_sql(out)
    }
}


#[cfg(test)]
mod tests {
    use diesel::serialize::Output;
    use diesel::sql_types::Integer;
    use diesel::sqlite::Sqlite;

    use super::*;

    #[test]
    fn id_to_sql() {
        for value in -10..=10 {
            let id = Id(value);

            let mut output = new_output();
            ToSql::<Integer, Sqlite>::to_sql(&id, &mut output).unwrap();

            assert_eq!(output, value.to_le_bytes());
        }
    }

    // #[test]
    // fn id_sql_roundtrip() {
    //     let id = Id::new();

    //     let mut bytes = new_output();
    //     ToSql::<Binary, Sqlite>::to_sql(&id, &mut bytes).unwrap();

    //     // FIXME: can't create a SqliteValue
    //     use diesel::sqlite::SqliteValue;
    //     let sql_value: SqliteValue = &bytes.into_inner();

    //     let out_id: Id = FromSql::<Binary, Sqlite>::from_sql(Some(sql_value)).unwrap();

    //     assert_eq!(out_id, id);
    // }

    fn new_output() -> Output<'static, Vec<u8>, Sqlite> {
        Output::new(Vec::new(), &())
    }
}
