#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_matrix() {
        // Use CARGO_MANIFEST_DIR to find data relative to workspace root
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let data_path = std::path::Path::new(manifest_dir).parent().unwrap().join("data/agaricus.txt.train");
        let config = format!(r#"{{"uri": "{}?format=libsvm", "silent": 1}}"#, data_path.display());

        let mut handle = std::ptr::null_mut();
        let config_cstr = std::ffi::CString::new(config).unwrap();
        let ret_val = unsafe { XGDMatrixCreateFromURI(config_cstr.as_ptr(), &mut handle) };
        assert_eq!(ret_val, 0);

        let mut num_rows = 0;
        let ret_val = unsafe { XGDMatrixNumRow(handle, &mut num_rows) };
        assert_eq!(ret_val, 0);
        assert_eq!(num_rows, 6513);

        let mut num_cols = 0;
        let ret_val = unsafe { XGDMatrixNumCol(handle, &mut num_cols) };
        assert_eq!(ret_val, 0);
        assert_eq!(num_cols, 127);

        let ret_val = unsafe { XGDMatrixFree(handle) };
        assert_eq!(ret_val, 0);
    }
}
