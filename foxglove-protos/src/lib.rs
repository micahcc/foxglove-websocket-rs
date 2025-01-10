#![allow(missing_docs)]
#![allow(unreachable_pub)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::too_long_first_doc_paragraph)]
include!(concat!(env!("OUT_DIR"), "/protos.rs"));

pub static DESCRIPTOR_POOL: once_cell::sync::Lazy<prost_reflect::DescriptorPool> =
    once_cell::sync::Lazy::new(|| {
        prost_reflect::DescriptorPool::decode(
            &include_bytes!(concat!(env!("OUT_DIR"), "/protos.desc"))[..],
        )
        .unwrap()
    });
