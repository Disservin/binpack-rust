#[cfg(target_arch = "x86_64")]
use raw_cpuid::CpuId;

#[cfg(target_arch = "x86_64")]
pub(crate) fn has_fast_bmi2() -> bool {
    if !std::is_x86_feature_detected!("bmi2") {
        return false;
    }

    !has_slow_bmi2()
}

#[cfg(target_arch = "x86_64")]
fn has_slow_bmi2() -> bool {
    let cpuid = CpuId::new();

    if !cpuid
        .get_vendor_info()
        .is_some_and(|vendor| matches!(vendor.as_str(), "AuthenticAMD" | "AMD" | "HygonGenuine"))
    {
        return false;
    }

    let Some(feature_info) = cpuid.get_feature_info() else {
        return false;
    };

    match (feature_info.family_id(), feature_info.model_id()) {
        // Excavator / bdver4.
        (0x15, 0x60..=0x7f) => true,
        // Zen, Zen+, Zen 2.
        (0x17, _) => true,
        // Hygon Dhyana.
        (0x18, _) => true,
        _ => false,
    }
}
