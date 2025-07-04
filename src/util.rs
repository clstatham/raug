//! Utility functions.

use std::sync::OnceLock;

use cpal::traits::{DeviceTrait, HostTrait};

use crate::{graph::runtime::AudioBackend, signal::SignalType};

/// Returns a list of available audio backends, as exposed by the `cpal` crate.
pub fn available_audio_backends() -> Vec<AudioBackend> {
    let mut backends = vec![];
    for host in cpal::available_hosts() {
        match host {
            #[cfg(all(target_os = "linux", feature = "jack"))]
            cpal::HostId::Jack => {
                backends.push(AudioBackend::Jack);
            }
            #[cfg(target_os = "linux")]
            cpal::HostId::Alsa => {
                backends.push(AudioBackend::Alsa);
            }
            #[cfg(target_os = "windows")]
            cpal::HostId::Wasapi => {
                backends.push(AudioBackend::Wasapi);
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    backends
}

/// Prints a list of available audio backends to the console.
pub fn list_audio_backends() {
    println!("Listing available backends:");
    for (i, backend) in available_audio_backends().into_iter().enumerate() {
        println!("  {}: {:?}", i, backend);
    }
}

/// Prints a list of available audio devices for the given backend to the console.
pub fn list_audio_devices(backend: AudioBackend) {
    println!("Listing devices for backend: {:?}", backend);
    let host = match backend {
        AudioBackend::Default => cpal::default_host(),
        #[cfg(all(target_os = "linux", feature = "jack"))]
        AudioBackend::Jack => cpal::host_from_id(cpal::HostId::Jack).unwrap(),
        #[cfg(target_os = "linux")]
        AudioBackend::Alsa => cpal::host_from_id(cpal::HostId::Alsa).unwrap(),
        #[cfg(target_os = "windows")]
        AudioBackend::Wasapi => cpal::host_from_id(cpal::HostId::Wasapi).unwrap(),
    };
    for (i, device) in host.output_devices().unwrap().enumerate() {
        println!("  {}: {:?}", i, device.name());
    }
}

#[inline]
pub fn interned_short_type_name<T: ?Sized>() -> &'static str {
    static NAME: OnceLock<String> = OnceLock::new();
    NAME.get_or_init(tynm::type_name::<T>)
}

#[inline]
#[track_caller]
pub fn assert_signals_compatible(a: &SignalType, b: &SignalType, op: impl Into<String>) {
    assert_eq!(
        a,
        b,
        "{}: incompatible signal types: {:?} vs {:?}",
        op.into(),
        a,
        b
    );
}
