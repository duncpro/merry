#[macro_export]
macro_rules! assert_matches {
    ($e:expr, $p:pat) => {
        let $p = $e 
        else { panic!("{:?} did not match pattern $p", $e); };
    }
}
