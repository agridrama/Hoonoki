use hnk_idl::ast::ContractBag;

pub fn has_must_reply(contracts: &ContractBag) -> bool {
    contracts
        .get("must_reply")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

pub fn timeout_after(contracts: &ContractBag) -> Option<String> {
    contracts
        .get("timeout")
        .and_then(|value| value.as_mapping())
        .and_then(|mapping| mapping.get("after"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}
