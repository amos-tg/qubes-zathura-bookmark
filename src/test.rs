use std::{
    path::PathBuf,
    fs::{
        read_dir,
        write,
        create_dir_all,
        remove_dir_all,
    },
    collections::HashMap,
};
use crate::{
    shared_fn::set_slice,
    shared_consts::DRes,
    client::StateFsTx,
};


const DIR_PATH: &str = "/tmp/qzb_testing_dir_89256";
struct FileCleaner;
impl Drop for FileCleaner {
    fn drop(&mut self) {
        let _ = remove_dir_all(DIR_PATH);
    }
}

/// this test is a little bit lazy, really I should
/// be including tests for recursive watching here
/// as well as dirname changes.
#[test]
fn state_fs_changes_test() -> DRes<()> {
    let _file_cleaner = FileCleaner;
    let dir_path = DIR_PATH;
    let fbase_path = format!("{dir_path}/test");
    let fcont_init = "Initial file value."; 
    let fcont_changed = "The file changed.";
    let mut fs_changes = HashMap::new();

    create_dir_all(dir_path)?;
    let mut init_list_cmp = vec!();
    for differ in 'a'..'f' {
        let fpath = PathBuf::from(format!("{fbase_path}{differ}"));
        write(&fpath, fcont_init)?;
        init_list_cmp.push(fpath);
    }

    let init_list =
        StateFsTx::state_fs_changes(&mut fs_changes, read_dir(dir_path)?)?;

    for file in init_list {
        assert!(
            init_list_cmp.contains(&file),
            "Error: init_list != init_list_cmp");
    } 

    let changed_path = PathBuf::from(format!("{fbase_path}d"));
    write(&changed_path, fcont_changed)?; 

    let changes_list_expected = vec!(changed_path);
    let changes_list = 
        StateFsTx::state_fs_changes(&mut fs_changes, read_dir(dir_path)?)?;
    for file in changes_list {
        assert!(
            changes_list_expected.contains(&file),
            "Error: changes_list != changes_list_expected");
    }

    return Ok(());
}

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
