use bindgen::Builder;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use git2::Repository;

static IREE_REPO: &str = "https://github.com/openxla/iree";

fn shallow_clone(path: &Path, repo: &str) -> Repository {
    let mut child = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            "--recurse-submodules",
            "--shallow-submodules",
            "-j10",
            repo,
            path.to_str().unwrap(),
        ])
        .spawn()
        .expect("failed to execute process");
    child.wait().unwrap();

    git2::Repository::open(path).unwrap()
}

/// use cached repo if it exists, otherwise clone it
fn get_repo(path: &Path, repo: &str) -> git2::Repository {
    println!("Checking for cached repo at: {}", path.to_str().unwrap());
    if path.exists() {
        git2::Repository::open(path).unwrap()
    } else {
        // shallow clone
        shallow_clone(path, repo)
    }
}

/// Clones the IREE repository and builds it.
fn clone_and_build_iree(out_dir: &Path) -> PathBuf {
    // clone IREE repo
    let iree_dir = out_dir.join("iree");
    let iree = get_repo(iree_dir.as_path(), IREE_REPO);

    // make build directory
    let mut iree_build_path = out_dir.join("iree-build");

    /*    cmake -G Ninja -B ../iree-build/  \
       -DCMAKE_BUILD_TYPE=RelWithDebInfo \
       -DIREE_ENABLE_ASSERTIONS=ON \
       -DIREE_ENABLE_SPLIT_DWARF=ON \
       -DIREE_ENABLE_THIN_ARCHIVES=ON \
       -DCMAKE_C_COMPILER=clang \
       -DCMAKE_CXX_COMPILER=clang++
           -DIREE_ENABLE_LLD=ON
    */

    // build iree
    cmake::Config::new(out_dir.join("iree"))
        .generator("Ninja")
        .define("CMAKE_BUILD_TYPE", "RelWithDebInfo")
        .define("IREE_ENABLE_ASSERTIONS", "ON")
        .define("IREE_BUILD_SAMPLES", "OFF")
        .define("IREE_ENABLE_SPLIT_DWARF", "ON")
        .define("IREE_ENABLE_THIN_ARCHIVES", "ON")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .define("CMAKE_C_COMPILER", "clang")
        .define("IREE_ENABLE_LLD", "ON")
        .define("IREE_TARGET_BACKEND_ROCM", "ON")
        .define("IREE_EXTERNAL_HAL_DRIVERS", "rocm")
        .define(
            "IREE_ROOT_DIR",
            out_dir
                .join("iree")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .out_dir(iree_build_path.clone())
        .build();

    // add library path to linker
    iree_build_path
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let iree_build_dir = clone_and_build_iree(out_path.as_path());
    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir
            .join("build/runtime/src/iree/runtime/")
            .to_str()
            .unwrap()
    );

    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir.join("build/lib").to_str().unwrap()
    );

    // add built third party libraries to linker
    // cpuinfo
    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir
            .join("build/third_party/cpuinfo/")
            .to_str()
            .unwrap()
    );

    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir
            .join("build/build_tools/third_party/flatcc/")
            .to_str()
            .unwrap()
    );

    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir
            .join("build/third_party/spirv_cross/")
            .to_str()
            .unwrap()
    );
    let iree_runtime_include_dir = out_path.join("iree/runtime/src");
    let iree_compiler_include_dir = out_path.join("iree/compiler/bindings/c/");

    println!("cargo:rustc-link-lib=iree_runtime_unified");
    //    println!("cargo:rustc-link-lib=IREECompiler");
    println!("cargo:rustc-link-lib=iree_compiler_bindings_c_loader");

    // third party libraries
    println!("cargo:rustc-link-lib=cpuinfo");
    println!("cargo:rustc-link-lib=flatcc_parsing");
    println!("cargo:rustc-link-lib=spirv-cross-core");

    println!("cargo:rustc-link-lib=stdc++");

    // gather all api headers we want
    let _iree_api_headers = ["iree/runtime/api.h", "iree/compiler/embedding_api.h"];

    let gen_header = (|include_dir: &PathBuf, header: &Path| {
        let header_out = Path::new(header)
            .to_str()
            .and_then(|s| s.strip_suffix(".h"))
            .and_then(|s| Some(format!("{}.rs", s)))
            .unwrap();

        let header_buf = include_dir.join(header);
        let header_path = header_buf.as_path();

        let dir = out_path.join(Path::new(header).parent().unwrap());

        if !dir.exists() {
            std::fs::create_dir_all(&dir).expect("Unable to create directory");
        }

        let bindings = Builder::default()
            .header(header_path.to_str().unwrap())
            .clang_arg(format!("-I{}", include_dir.to_str().unwrap()))
            .default_enum_style(bindgen::EnumVariation::NewType {
                is_bitfield: true,
                is_global: true,
            })
            .generate_inline_functions(false)
            .derive_default(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join(header_out))
            .expect("Couldn't write bindings!");
    });

    gen_header(
        &out_path.join("iree/runtime/src"),
        Path::new("iree/runtime/api.h"),
        /*         Path::new("iree/runtime/api.h"),
         */
    );

    gen_header(
        &out_path.join("iree/runtime/src"),
        Path::new("iree/base/api.h"),
        /*         Path::new("iree/base/time.h"),
         */
    );
    gen_header(
        &out_path.join("iree/compiler/bindings/c/"),
        Path::new("iree/compiler/embedding_api.h"),
        /*         Path::new("iree/compiler/embedding_api.h"),
         */
    );

    gen_header(
        &out_path.join("iree/compiler/bindings/c/"),
        Path::new("iree/compiler/loader.h"),
        /*         Path::new("iree/compiler/loader.h"),
         */
    );

    println!("cargo:rerun-if-changed=build.rs");
}
