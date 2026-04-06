// UnTrickyStore - i18n via gettext
//
// Embeds compiled .mo catalogs, selects locale from Android system props.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use gettext::Catalog;
use std::sync::OnceLock;

static CATALOG: OnceLock<Option<Catalog>> = OnceLock::new();

/// Embedded zh_CN.mo (compiled from i18n/zh_CN.po by build.rs)
const ZH_CN_MO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/zh_CN.mo"));

/// Initialize the i18n catalog based on device locale.
pub fn init() {
    CATALOG.get_or_init(|| {
        if is_chinese() {
            Catalog::parse(ZH_CN_MO).ok()
        } else {
            None // English = passthrough (msgid returned as-is)
        }
    });
}

/// Translate a message. Returns msgid verbatim if no translation found.
pub fn t(msgid: &str) -> String {
    init();
    match CATALOG.get() {
        Some(Some(cat)) => cat.gettext(msgid).to_string(),
        _ => msgid.to_string(),
    }
}

/// Translate with one string argument (replaces first `%s`).
pub fn t_fmt(msgid: &str, arg: &str) -> String {
    t(msgid).replacen("%s", arg, 1)
}

fn is_chinese() -> bool {
    let locale = crate::common::getprop("persist.sys.locale");
    let product = crate::common::getprop("ro.product.locale");
    locale.contains("zh") || product.contains("zh")
}
