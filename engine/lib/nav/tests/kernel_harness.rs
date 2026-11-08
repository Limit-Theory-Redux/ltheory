//! Self-tests of the kernel harness, run against fixture kernels in `tests/fixtures/`.

mod common;

use common::KernelVm;
use mlua::Function;

fn fixtures_dir() -> String {
    format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR")).replace('\\', "/")
}

fn vm() -> KernelVm {
    KernelVm::new(Some(&fixtures_dir())).expect("create the kernel state")
}

#[test]
fn kernel_computes_over_buffers() {
    let vm = vm();
    let kernel = vm.require("KernelFixtures.Axpy").expect("load the kernel");
    let x = vm.buffer(&[1.0, 2.0, 3.0]).unwrap();
    let y = vm.buffer(&[10.0, 20.0, 30.0]).unwrap();
    let out = vm.buffer(&[0.0; 3]).unwrap();

    kernel
        .get::<Function>("run")
        .unwrap()
        .call::<()>((2.0, &x, &y, &out, 3))
        .expect("run the kernel");

    assert_eq!(vm.read_buffer(&out, 3).unwrap(), vec![12.0, 24.0, 36.0]);
}

#[test]
fn kernel_leaves_elements_beyond_n_untouched() {
    let vm = vm();
    let kernel = vm.require("KernelFixtures.Axpy").unwrap();
    let x = vm.buffer(&[1.0, 1.0]).unwrap();
    let y = vm.buffer(&[1.0, 1.0]).unwrap();
    let out = vm.buffer(&[-1.0, -1.0]).unwrap();

    kernel
        .get::<Function>("run")
        .unwrap()
        .call::<()>((1.0, &x, &y, &out, 1))
        .unwrap();

    assert_eq!(vm.read_buffer(&out, 2).unwrap(), vec![2.0, -1.0]);
}

#[test]
fn missing_module_names_the_module() {
    let error = vm().require("KernelFixtures.DoesNotExist").unwrap_err();
    assert!(
        error.to_string().contains("KernelFixtures.DoesNotExist"),
        "error should name the module: {error}"
    );
}

#[test]
fn reading_an_undefined_global_fails() {
    let vm = vm();
    let kernel = vm.require("KernelFixtures.ReadsGlobal").unwrap();
    let error = kernel
        .get::<Function>("run")
        .unwrap()
        .call::<()>(())
        .unwrap_err();
    assert!(
        error.to_string().contains("undefined global read: Config"),
        "unexpected error: {error}"
    );
}

#[test]
fn writing_a_global_while_loading_fails() {
    let error = vm().require("KernelFixtures.WritesGlobal").unwrap_err();
    assert!(
        error.to_string().contains("global write: leaked"),
        "unexpected error: {error}"
    );
}

#[test]
fn repository_script_directory_is_on_the_module_path() {
    let error = KernelVm::new(None)
        .unwrap()
        .require("Shared.Helpers.DoesNotExist")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("script/Shared/Helpers/DoesNotExist.lua"),
        "the search should have covered script/: {error}"
    );
}
