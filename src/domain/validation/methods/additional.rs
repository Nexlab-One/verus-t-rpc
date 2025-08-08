use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel, ParameterValidationRule, ParameterType, ValidationConstraint};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_additional_methods(registry: &mut MethodRegistry) {
    let additional_methods = vec![
        ("getbestblockhash", "Get best block hash", true, vec![], vec![]),
        ("getblockhashes", "Get block hashes", true, vec![], vec![
            ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ("count", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
        ]),
        ("getblocksubsidy", "Get block subsidy", true, vec![], vec![
            ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
        ]),
        ("getblocktemplate", "Get block template", true, vec![], vec![
            ("template_request", ParameterType::Object, true, vec![]),
        ]),
        ("getchaintips", "Get chain tips", true, vec![], vec![]),

        ("getaddressbalance", "Get address balance", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),
        ("getaddressutxos", "Get address UTXOs", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),
        ("getaddressdeltas", "Get address deltas", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),
        ("getaddresstxids", "Get address transaction IDs", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),
        ("getaddressmempool", "Get address mempool", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),

        ("getrawmempool", "Get raw mempool", true, vec![], vec![]),
        ("gettxout", "Get transaction output", true, vec![], vec![
            ("txid", ParameterType::String, true, vec![ValidationConstraint::MinLength(64)]),
            ("n", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ("include_mempool", ParameterType::Boolean, false, vec![]),
        ]),
        ("gettxoutsetinfo", "Get transaction output set info", true, vec![], vec![]),
        ("getspentinfo", "Get spent info", true, vec![], vec![
            ("txid", ParameterType::Object, true, vec![]),
        ]),

        ("getcurrencystate", "Get currency state", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("fromcurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("tocurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getcurrencyconverters", "Get currency converters", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("fromcurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("tocurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getcurrencytrust", "Get currency trust", true, vec![], vec![
            ("addresses", ParameterType::Array, true, vec![]),
        ]),
        ("getinitialcurrencystate", "Get initial currency state", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),

        ("getidentitieswithaddress", "Get identities with address", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),
        ("getidentitieswithrevocation", "Get identities with revocation", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),
        ("getidentitieswithrecovery", "Get identities with recovery", true, vec![], vec![
            ("addresses", ParameterType::Object, true, vec![]),
        ]),
        ("getidentitytrust", "Get identity trust", true, vec![], vec![
            ("addresses", ParameterType::Array, true, vec![]),
        ]),
        ("getidentitycontent", "Get identity content", true, vec![], vec![
            ("identity", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ("txproofheight", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ("txproof", ParameterType::Boolean, false, vec![]),
            ("txproofheight", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ("content", ParameterType::String, false, vec![]),
            ("contentproof", ParameterType::Boolean, false, vec![]),
        ]),

        ("createmultisig", "Create multi-signature", true, vec![], vec![
            ("nrequired", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
            ("keys", ParameterType::Array, true, vec![]),
        ]),
        ("createrawtransaction", "Create raw transaction", true, vec![], vec![
            ("inputs", ParameterType::Array, true, vec![]),
            ("outputs", ParameterType::Object, true, vec![]),
            ("locktime", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ("expiryheight", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
        ]),
        ("decoderawtransaction", "Decode raw transaction", true, vec![], vec![
            ("hexstring", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("iswitness", ParameterType::Boolean, false, vec![]),
        ]),
        ("decodescript", "Decode script", true, vec![], vec![
            ("hexstring", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("iswitness", ParameterType::Boolean, false, vec![]),
        ]),
        ("estimatefee", "Estimate fee", true, vec![], vec![
            ("nblocks", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
        ]),
        ("estimatepriority", "Estimate priority", true, vec![], vec![
            ("nblocks", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
        ]),
        ("verifymessage", "Verify message", true, vec![], vec![
            ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("signature", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("message", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("checkexpiry", ParameterType::Boolean, false, vec![]),
        ]),
        ("verifyhash", "Verify hash", true, vec![], vec![
            ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("signature", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("hash", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("checkexpiry", ParameterType::Boolean, false, vec![]),
        ]),
        ("verifysignature", "Verify signature", true, vec![], vec![
            ("signature", ParameterType::Object, true, vec![]),
        ]),
        ("hashdata", "Hash data", true, vec![], vec![
            ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("hexstring", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("messagetype", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("convertpassphrase", "Convert passphrase", true, vec![], vec![
            ("passphrase", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getvdxfid", "Get VDXF ID", true, vec![], vec![
            ("vdxfkey", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("vdxfobj", ParameterType::Object, false, vec![]),
        ]),
        ("getlastimportfrom", "Get last import from", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getlaunchinfo", "Get launch info", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getpendingtransfers", "Get pending transfers", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getreservedeposits", "Get reserved deposits", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getsaplingtree", "Get Sapling tree", true, vec![], vec![
            ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
        ]),
        ("getexports", "Get exports", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ("count", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
        ]),
        ("getnotarizationdata", "Get notarization data", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("getoffers", "Get offers", true, vec![], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("fromcurrency", ParameterType::Boolean, false, vec![]),
            ("tocurrency", ParameterType::Boolean, false, vec![]),
        ]),
        ("makeOffer", "Create marketplace offer", false, vec!["write".to_string()], vec![
            ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("offer", ParameterType::Object, true, vec![]),
            ("fromcurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("tocurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("amount", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ("price", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ("expiry", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
        ]),
        ("z_getnewaddress", "Get new Z-address", true, vec![], vec![
            ("type", ParameterType::String, false, vec![ValidationConstraint::Enum(vec!["sprout".to_string(), "sapling".to_string(), "orchard".to_string()])]),
        ]),
        ("z_listaddresses", "List Z-addresses", true, vec![], vec![]),
        ("z_getbalance", "Get Z-address balance", true, vec![], vec![
            ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("minconf", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
        ]),
        ("z_sendmany", "Send to multiple Z-addresses", false, vec!["write".to_string()], vec![
            ("fromaddress", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("amounts", ParameterType::Array, true, vec![]),
            ("minconf", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ("fee", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
        ]),
        ("z_shieldcoinbase", "Shield coinbase funds to Z-address", false, vec!["write".to_string()], vec![
            ("fromaddress", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("toaddress", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("fee", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ("limit", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
        ]),
        ("z_validateaddress", "Validate Z-address", true, vec![], vec![
            ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("z_viewtransaction", "View Z-transaction details", true, vec![], vec![
            ("txid", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("z_exportkey", "Export Z-address private key", false, vec!["write".to_string()], vec![
            ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("z_importkey", "Import Z-address private key", false, vec!["write".to_string()], vec![
            ("zkey", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("rescan", ParameterType::String, false, vec![ValidationConstraint::Enum(vec!["yes".to_string(), "no".to_string(), "whenkeyisnew".to_string()])]),
        ]),
        ("z_exportviewingkey", "Export Z-address viewing key", false, vec!["write".to_string()], vec![
            ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
        ]),
        ("z_importviewingkey", "Import Z-address viewing key", false, vec!["write".to_string()], vec![
            ("vkey", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ("rescan", ParameterType::String, false, vec![ValidationConstraint::Enum(vec!["yes".to_string(), "no".to_string(), "whenkeyisnew".to_string()])]),
        ]),
        ("listcurrencies", "List currencies", true, vec![], vec![
            ("currency", ParameterType::Object, false, vec![]),
            ("start", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ("count", ParameterType::Number, false, vec![ValidationConstraint::MinValue(1.0)]),
        ]),
        ("coinsupply", "Get coin supply", true, vec![], vec![]),
        ("getbestproofroot", "Get best proof root", true, vec![], vec![
            ("proofroot", ParameterType::Object, true, vec![]),
        ]),
    ];

    for (name, description, read_only, permissions, param_rules) in additional_methods {
        let mut parameter_rules = Vec::new();
        for (i, (param_name, param_type, required, constraints)) in param_rules.iter().enumerate() {
            parameter_rules.push(ParameterValidationRule {
                index: i,
                name: param_name.to_string(),
                param_type: param_type.clone(),
                required: *required,
                constraints: constraints.clone(),
                default_value: None,
            });
        }

        registry.register_method(RpcMethodDefinition {
            name: name.to_string(),
            description: description.to_string(),
            read_only,
            required_permissions: permissions,
            parameter_rules,
            security_level: if read_only { SecurityLevel::Low } else { SecurityLevel::Medium },
            enabled: true,
        });
    }
}


