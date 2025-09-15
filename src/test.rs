use crate::shared_fn::set_slice;

#[test]
fn set_slice_test() {
    const SET_ERR: &str = 
        "TEST: set_slice_test: Failed to set hello world! to HELLO WORLD!";
    const NUM_BYTES_ERR: &str = 
        "TEST: set_slice_test: Failed to return accurate number of bytes read"; 

    let mut test_init = vec!();
    test_init.extend_from_slice(b"hello world!");

    let mut exp_ret = vec!();
    exp_ret.extend_from_slice(b"HELLO WORLD!");

    let nb = set_slice(&mut test_init, &exp_ret);
    assert_eq!(test_init, exp_ret, "{}", SET_ERR);
    assert_eq!(nb, exp_ret.len(), "{}", NUM_BYTES_ERR);
}
