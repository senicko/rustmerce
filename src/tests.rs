use rocket::local::blocking::Client;

#[test]
fn count_test() {
    let client = Client::tracked(super::rocket()).unwrap();

    fn get_count(client: &Client) -> usize {
        let response = client.get("/count").dispatch().into_string().unwrap();
        let count = response.split(" ").last().unwrap();
        count.parse().unwrap()
    }

    for i in 1..128 {
        assert_eq!(get_count(&client), i)
    }
}
