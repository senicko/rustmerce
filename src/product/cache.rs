// #[derive(Clone)]
// pub struct ProductCache<'a> {
//     pub redis_conn: &'a mut redis::Connection,
// }

// impl<'a> ProductCache<'a> {
//     pub fn new(redis_conn: &'a mut redis::Connection) -> Self {
//         ProductCache { redis_conn }
//     }

//     fn set(&'a mut self) -> Result<(), redis::RedisError> {
//         let result: String = redis::cmd("JSON.SET")
//             .arg(&["products", "$", "{\"name\": \"test\"}"])
//             .query(self.redis_conn)?;

//         println!("{result}");
//         Ok(())
//     }

//     fn all(&mut self) -> Result<Option<String>, redis::RedisError> {
//         let result: String = redis::cmd("JSON.GET")
//             .arg("products")
//             .query(&mut self.redis_conn)?;

//         println!("{result}");
//         Ok(Some(result))
//     }
// }
