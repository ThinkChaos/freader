use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql};
use diesel::sql_types::Binary;
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

mod executor;
mod helper;

pub use executor::Executor;
pub use helper::Helper;


#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[derive(Hash, Eq, PartialEq)] // for diesel
#[derive(AsExpression, FromSqlRow)]
#[sql_type = "Binary"]
pub struct Id(Uuid);

impl Id {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Id(Uuid::new_v4())
    }

    #[cfg(test)]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl<DB> FromSql<Binary, DB> for Id
where
    DB: Backend,
    *const [u8]: FromSql<Binary, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        // the pointer is only valid until the end of this function
        let slice_ptr = <*const [u8] as FromSql<Binary, DB>>::from_sql(bytes)?;
        let bytes = unsafe { &*slice_ptr };

        let uuid = uuid::Builder::from_slice(bytes)?
            .set_version(uuid::Version::Random) // v4
            .build(); // copies bytes from `string` which makes its use safe

        Ok(Id(uuid))
    }
}

impl<DB> ToSql<Binary, DB> for Id
where
    DB: Backend,
    [u8]: ToSql<Binary, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
        self.0.as_bytes().to_sql(out)
    }
}


#[cfg(test)]
mod tests {
    use diesel::serialize::Output;
    use diesel::sql_types::Binary;
    use diesel::sqlite::Sqlite;

    use super::*;

    #[test]
    fn id_to_sql() {
        let id = Id::new();

        let mut bytes = new_output();
        ToSql::<Binary, Sqlite>::to_sql(&id, &mut bytes).unwrap();

        assert_eq!(bytes, id.as_uuid().as_bytes());
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
