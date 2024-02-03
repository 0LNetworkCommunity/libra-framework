use move_command_line_common::address::NumericalAddress;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

// ===============================================================================================
// Legacy Named Addresses

// Some older Move tests work directly on sources, skipping the package system. For those
// we define the relevant address aliases here.

static NAMED_ADDRESSES: Lazy<BTreeMap<String, NumericalAddress>> = Lazy::new(|| {
    let mut result = BTreeMap::new();
    let zero = NumericalAddress::parse_str("0x0").unwrap();
    let one = NumericalAddress::parse_str("0x1").unwrap();
    let three = NumericalAddress::parse_str("0x3").unwrap();
    let four = NumericalAddress::parse_str("0x4").unwrap();
    let resources = NumericalAddress::parse_str("0xA550C18").unwrap();
    result.insert("std".to_owned(), one);
    result.insert("diem_std".to_owned(), one);
    result.insert("diem_framework".to_owned(), one);
    result.insert("diem_token".to_owned(), three);
    result.insert("diem_token_objects".to_owned(), four);
    result.insert("core_resources".to_owned(), resources);
    result.insert("vm_reserved".to_owned(), zero);
    result.insert("ol_framework".to_owned(), one); /////// 0L /////////
    result
});

pub fn named_addresses() -> &'static BTreeMap<String, NumericalAddress> {
    &NAMED_ADDRESSES
}
