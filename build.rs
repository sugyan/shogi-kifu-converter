fn main() {
    protobuf_codegen::Codegen::new()
        // Use `protoc` parser, optional.
        .protoc()
        // Use `protoc-bin-vendored` bundled protoc command, optional.
        .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
        // All inputs and imports from the inputs must reside in `includes` directories.
        .includes(&["src"])
        // Inputs must reside in some of include paths.
        .input("src/protos/pkf.proto")
        // Specify output directory relative to Cargo output directory.
        .cargo_out_dir("pkf")
        .run_from_script();
}
