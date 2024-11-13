fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .compile_protos(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    // fs::remove_file("src/pb/google.protobuf.rs").unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}
