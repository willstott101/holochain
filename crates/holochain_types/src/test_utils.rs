//! Some common testing helpers.

use crate::dna::zome::WasmZome;
use crate::dna::DnaDef;
use crate::dna::DnaFile;
use crate::dna::JsonProperties;
use crate::element::SignedHeaderHashedExt;
use crate::fixt::*;
use crate::prelude::*;
use crate::Timestamp;
use crate::{dna::wasm::DnaWasm, EntryHashed};
use holochain_zome_types::prelude::*;
use std::path::PathBuf;

pub use holochain_zome_types::test_utils::*;

#[derive(Serialize, Deserialize, SerializedBytes)]
struct FakeProperties {
    test: String,
}

/// simple DnaWasm fixture
pub fn fake_dna_wasm() -> DnaWasm {
    DnaWasm::from(vec![0_u8])
}

/// A fixture example dna for unit testing.
pub fn fake_dna_file(uuid: &str) -> DnaFile {
    fake_dna_zomes(uuid, vec![("test".into(), vec![].into())])
}

/// A fixture example dna for unit testing.
pub fn fake_dna_zomes(uuid: &str, zomes: Vec<(ZomeName, DnaWasm)>) -> DnaFile {
    let mut dna = DnaDef {
        name: "test".to_string(),
        properties: JsonProperties::new(serde_json::json!({"p": "hi"}))
            .try_into()
            .unwrap(),
        uuid: uuid.to_string(),
        zomes: Vec::new(),
    };
    tokio_safe_block_on::tokio_safe_block_forever_on(async move {
        let mut wasm_code = Vec::new();
        for (zome_name, wasm) in zomes {
            let wasm = crate::dna::wasm::DnaWasmHashed::from_content(wasm).await;
            let (wasm, wasm_hash) = wasm.into_inner();
            dna.zomes.push((zome_name, WasmZome { wasm_hash }.into()));
            wasm_code.push(wasm);
        }
        DnaFile::new(dna, wasm_code).await
    })
    .unwrap()
}

/// Save a Dna to a file and return the path and tempdir that contains it
pub async fn write_fake_dna_file(dna: DnaFile) -> anyhow::Result<(PathBuf, tempdir::TempDir)> {
    let tmp_dir = tempdir::TempDir::new("fake_dna")?;
    let mut path: PathBuf = tmp_dir.path().into();
    path.push("test-dna.dna.gz");
    tokio::fs::write(path.clone(), dna.to_file_content().await?).await?;
    Ok((path, tmp_dir))
}

/// Re-export for convenience [ TK-06690 ]
pub use holochain_zome_types::test_utils::fake_cell_id;

/// Keeping with convention if Alice is pubkey 1
/// and bob is pubkey 2 the this helps make test
/// logging easier to read.
pub fn which_agent(key: &AgentPubKey) -> String {
    let key = key.to_string();
    let alice = fake_agent_pubkey_1().to_string();
    let bob = fake_agent_pubkey_2().to_string();
    if key == alice {
        return "alice".to_string();
    }
    if key == bob {
        return "bob".to_string();
    }
    key
}

/// A fixture CapSecret for unit testing.
pub fn fake_cap_secret() -> CapSecret {
    [0; CAP_SECRET_BYTES].into()
}

/// Create a fake SignedHeaderHashed and EntryHashed pair with random content
pub async fn fake_unique_element(
    keystore: &KeystoreSender,
    agent_key: AgentPubKey,
    visibility: EntryVisibility,
) -> anyhow::Result<(SignedHeaderHashed, EntryHashed)> {
    let content: SerializedBytes =
        UnsafeBytes::from(nanoid::nanoid!().as_bytes().to_owned()).into();
    let entry = EntryHashed::from_content_sync(Entry::App(content.try_into().unwrap()));
    let app_entry_type = AppEntryTypeFixturator::new(visibility).next().unwrap();
    let header_1 = Header::Create(Create {
        author: agent_key,
        timestamp: Timestamp::now().into(),
        header_seq: 0,
        prev_header: fake_header_hash(1),

        entry_type: EntryType::App(app_entry_type),
        entry_hash: entry.as_hash().to_owned(),
    });

    Ok((
        SignedHeaderHashed::new(&keystore, HeaderHashed::from_content_sync(header_1)).await?,
        entry,
    ))
}
