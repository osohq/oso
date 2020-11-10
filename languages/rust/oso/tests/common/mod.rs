#![allow(dead_code)]
use oso::Oso;

pub struct OsoTest {
    pub oso: Oso,
}

impl OsoTest {
    pub fn new() -> Self {
        Self { oso: Oso::new() }
    }

    #[track_caller]
    pub fn load_str(&mut self, policy: &str) {
        self.oso.load_str(policy).unwrap();
    }

    #[track_caller]
    pub fn load_file(&mut self, here: &str, name: &str) -> oso::Result<()> {
        // hack because `file!()` starts from workspace root
        // https://github.com/rust-lang/cargo/issues/3946
        let folder = std::path::PathBuf::from(&here.replace("languages/rust/oso/", ""));
        let mut file = folder.parent().unwrap().to_path_buf();
        file.push(name);
        println!("{:?}", file);
        self.oso.load_file(file.to_str().unwrap())
    }

    #[track_caller]
    pub fn query(&mut self, q: &str) -> Vec<oso::ResultSet> {
        let results = self.oso.query(q).unwrap();
        let mut result_vec = vec![];
        for r in results {
            result_vec.push(r.expect("result is an error"))
        }
        result_vec
    }

    #[track_caller]
    pub fn query_err(&mut self, q: &str) -> String {
        let mut results = self.oso.query(q).unwrap();
        let err = results
            .next()
            .unwrap()
            .expect_err("query should return an error");
        err.to_string()
    }

    #[track_caller]
    pub fn qvar<T: oso::FromPolar>(&mut self, q: &str, var: &str) -> Vec<T> {
        let res = self.query(q);
        res.into_iter()
            .map(|set| {
                tracing::trace!("bindings {:?}", set.keys().collect::<Vec<_>>());
                set.get_typed(var).unwrap_or_else(|_| {
                    panic!(
                        "query: '{}', no binding for '{}' with type {}",
                        q,
                        var,
                        std::any::type_name::<T>()
                    )
                })
            })
            .collect()
    }

    #[track_caller]
    pub fn qeval(&mut self, q: &str) {
        let mut results = self.oso.query(q).unwrap();
        results
            .next()
            .expect("Query should have at least one result.")
            .unwrap();
    }

    #[track_caller]
    pub fn qnull(&mut self, q: &str) {
        let mut results = self.oso.query(q).unwrap();
        assert!(results.next().is_none(), "Query shouldn't have any results");
    }

    #[track_caller]
    pub fn qvar_one<T>(&mut self, q: &str, var: &str, expected: T)
    where
        T: oso::FromPolar + PartialEq<T> + std::fmt::Debug,
    {
        let mut res = self.qvar::<T>(q, var);
        assert_eq!(res.len(), 1, "expected exactly one result");
        assert_eq!(res.pop().unwrap(), expected);
    }
}

/// Pretest setup.
pub fn setup() {
    let _ = tracing_subscriber::fmt::try_init();
}
