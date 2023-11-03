#![allow(clippy::needless_range_loop)]
use diem_framework::ReleasePackage;
use diem_types::account_address::AccountAddress;
use move_model::{code_writer::CodeWriter, emit, emitln, model::Loc};
use std::path::PathBuf;

// TODO 0L: make public
fn generate_blob(writer: &CodeWriter, data: &[u8]) {
    emitln!(writer, "vector[");
    writer.indent();
    for (i, b) in data.iter().enumerate() {
        if (i + 1) % 20 == 0 {
            emitln!(writer);
        }
        emit!(writer, "{}u8,", b);
    }
    emitln!(writer);
    writer.unindent();
    emit!(writer, "]")
}

pub fn libra_author_script_file(
    //////// 0L //////// turn an MRB into a script proposal
    release_package: &ReleasePackage,
    for_address: AccountAddress,
    out: PathBuf,
    next_execution_hash: Vec<u8>, // metadata: Option<PackageMetadata>,
                                  // is_testnet: bool,
                                  // is_multi_step: bool,
                                  // next_execution_hash: Vec<u8>,
) -> anyhow::Result<()> {
    println!("autogenerating .move governance script file");
    let metadata = &release_package.metadata;

    let writer = CodeWriter::new(Loc::default());
    emitln!(
        writer,
        "// Upgrade proposal for package `{}`\n",
        metadata.name
    );
    emitln!(
        writer,
        "// Framework commit hash: {}\n// Builder commit hash: {}\n",
        diem_build_info::get_git_hash(),
        diem_build_info::get_git_hash(),
    );
    emitln!(
        writer,
        "// Next step script hash: {}\n",
        hex::encode(&next_execution_hash),
    );
    emitln!(writer, "// source digest: {}", metadata.source_digest);
    emitln!(writer, "script {");
    writer.indent();
    emitln!(writer, "use std::vector;");
    emitln!(writer, "use diem_framework::diem_governance;");
    emitln!(writer, "use diem_framework::code;\n");
    emitln!(writer, "use diem_framework::version;\n");


    emitln!(writer, "fun main(proposal_id: u64){");
    writer.indent();
    // This is the multi step proposal, needs a next hash even if it a single step and thus an empty vec.
    generate_next_execution_hash_blob(&writer, for_address, next_execution_hash);

    emitln!(writer, "let code = vector::empty();");

    let code = release_package.code();
    for i in 0..code.len() {
        emitln!(writer, "let chunk{} = ", i);
        generate_blob(&writer, code[i]);
        emitln!(writer, ";");
        emitln!(writer, "vector::push_back(&mut code, chunk{});", i);
    }

    // The package metadata can be larger than 64k, which is the max for Move constants.
    // We therefore have to split it into chunks. Three chunks should be large enough
    // to cover any current and future needs. We then dynamically append them to obtain
    // the result.
    let mut metadata = bcs::to_bytes(&metadata)?;
    let chunk_size = (u16::MAX / 2) as usize;
    let num_of_chunks = (metadata.len() / chunk_size) + 1;

    for i in 1..num_of_chunks + 1 {
        let to_drain = if i == num_of_chunks {
            metadata.len()
        } else {
            chunk_size
        };
        let chunk = metadata.drain(0..to_drain).collect::<Vec<_>>();
        emit!(writer, "let chunk{} = ", i);
        generate_blob(&writer, &chunk);
        emitln!(writer, ";")
    }

    for i in 2..num_of_chunks + 1 {
        emitln!(writer, "vector::append(&mut chunk1, chunk{});", i);
    }

    emitln!(
        writer,
        "code::publish_package_txn(&framework_signer, chunk1, code);"
    );

    emitln!(
        writer,
        "version::upgrade_set_git(&framework_signer, x\"{}\")",
        diem_build_info::get_git_hash()
    );


    writer.unindent();
    emitln!(writer, "}");
    writer.unindent();
    emitln!(writer, "}");
    writer.process_result(|s| std::fs::write(&out, s))?;
    Ok(())
}

fn generate_next_execution_hash_blob(
    writer: &CodeWriter,
    for_address: AccountAddress,
    next_execution_hash: Vec<u8>,
) {
    if next_execution_hash == "vector::empty<u8>()".as_bytes() {
        emitln!(
                writer,
                "let framework_signer = diem_governance::resolve_multi_step_proposal(proposal_id, @{}, {});\n",
                for_address,
                "vector::empty<u8>()",
            );
    } else {
        emitln!(
            writer,
            "let framework_signer = diem_governance::resolve_multi_step_proposal("
        );
        writer.indent();
        emitln!(writer, "proposal_id,");
        emitln!(writer, "@{},", for_address);
        emit!(writer, "vector[");
        for (_, b) in next_execution_hash.iter().enumerate() {
            emit!(writer, "{}u8,", b);
        }
        emitln!(writer, "],");
        writer.unindent();
        emitln!(writer, ");");
    }
}
