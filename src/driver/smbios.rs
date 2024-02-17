use crate::flow::FlowSerdeError;
use alloc::vec::IntoIter;
use core::mem::discriminant;
use itertools::Itertools;
use smbioslib::*;
use x86_64::{PhysAddr, VirtAddr};
use distros_memory::translate_kernel;

struct MemoryMapperImpl {}

impl MemoryMapper for MemoryMapperImpl {
    #[inline]
    fn map_block(&mut self, addr: PhysAddr, _size: usize) -> VirtAddr {
        translate_kernel(addr)
    }
}

fn add_struct(pos: usize, s: &DefinedStruct<'_>) -> Result<(), FlowSerdeError> {
    match s {
        DefinedStruct::SystemReset(s) => register!(serial "/dev/smbios/system_reset" => s)?,
        DefinedStruct::AdditionalInformation(s) => {
            register!(serial "/dev/smbios/additional_info" => s)?
        }
        DefinedStruct::BaseBoardInformation(s) => register!(serial "/dev/smbios/base_board" => s)?,
        DefinedStruct::BisEntryPoint(s) => register!(serial "/dev/smbios/bis_entry_point" => s)?,
        DefinedStruct::BuiltInPointingDevice(s) => {
            register!(serial format!("/dev/smbios/built_in_pointing_device/{}", pos) => s)?
        }
        DefinedStruct::CacheInformation(s) => {
            register!(serial format!("/dev/smbios/cache_info/{}", pos) => s)?
        }
        DefinedStruct::CoolingDevice(s) => {
            register!(serial format!("/dev/smbios/cooling_device/{}", pos) => s)?
        }
        DefinedStruct::ElectricalCurrentProbe(s) => {
            register!(serial format!("/dev/smbios/electrical_current_probe/{}", pos) => s)?
        }
        DefinedStruct::EventLog(s) => {
            register!(serial format!("/dev/smbios/event_log/{}", pos) => s)?
        }
        DefinedStruct::GroupAssociations(s) => {
            register!(serial format!("/dev/smbios/group_associations/{}", pos) => s)?
        }
        DefinedStruct::HardwareSecurity(s) => {
            register!(serial format!("/dev/smbios/hardware_security/{}", pos) => s)?
        }
        DefinedStruct::Information(s) => register!(serial "/dev/smbios/info" => s)?,
        DefinedStruct::IpmiDeviceInformation(s) => {
            register!(serial format!("/dev/smbios/ipmi_device/{}", pos) => s)?
        }
        DefinedStruct::LanguageInformation(s) => {
            register!(serial format!("/dev/smbios/language/{}", pos) => s)?
        }
        DefinedStruct::ManagementControllerHostInterface(s) => {
            register!(serial format!("/dev/smbios/management_controller_host/{}", pos) => s)?
        }
        DefinedStruct::ManagementDevice(s) => {
            register!(serial format!("/dev/smbios/management_device/{}", pos) => s)?
        }
        DefinedStruct::ManagementDeviceComponent(s) => {
            register!(serial format!("/dev/smbios/management_device_component/{}", pos) => s)?
        }
        DefinedStruct::ManagementDeviceThresholdData(s) => {
            register!(serial format!("/dev/smbios/management_device_threshold_data/{}", pos) => s)?
        }
        DefinedStruct::MemoryArrayMappedAddress(s) => {
            register!(serial format!("/dev/smbios/memory_array_mapped_address/{}", pos) => s)?
        }
        DefinedStruct::MemoryChannel(s) => {
            register!(serial format!("/dev/smbios/memory_channel/{}", pos) => s)?
        }
        DefinedStruct::MemoryControllerInformation(s) => {
            register!(serial format!("/dev/smbios/memory_controller/{}", pos) => s)?
        }
        DefinedStruct::MemoryDevice(s) => {
            register!(serial format!("/dev/smbios/memory_device/{}", pos) => s)?
        }
        DefinedStruct::MemoryDeviceMappedAddress(s) => {
            register!(serial format!("/dev/smbios/memory_device_mapped_address/{}", pos) => s)?
        }
        DefinedStruct::MemoryErrorInformation32Bit(s) => {
            register!(serial format!("/dev/smbios/memory_error_32bit/{}", pos) => s)?
        }
        DefinedStruct::MemoryErrorInformation64Bit(s) => {
            register!(serial format!("/dev/smbios/memory_error_64bit/{}", pos) => s)?
        }
        DefinedStruct::MemoryModuleInformation(s) => {
            register!(serial format!("/dev/smbios/memory_module/{}", pos) => s)?
        }
        DefinedStruct::OemStrings(s) => {
            register!(serial format!("/dev/smbios/oem_string/{}", pos) => s)?
        }
        DefinedStruct::OnBoardDeviceInformation(s) => {
            register!(serial "/dev/smbios/onboard_devices" => s)?
        }
        DefinedStruct::OnboardDevicesExtendedInformation(s) => {
            register!(serial "/dev/smbios/onboard_device_extended" => s)?
        }
        DefinedStruct::OutOfBandRemoteAccess(s) => {
            register!(serial format!("/dev/smbios/out_of_band_remote_access/{}", pos) => s)?
        }
        DefinedStruct::PhysicalMemoryArray(s) => {
            register!(serial format!("/dev/smbios/physical_memory_array/{}", pos) => s)?
        }
        DefinedStruct::PortableBattery(s) => {
            register!(serial format!("/dev/smbios/portable_battery/{}", pos) => s)?
        }
        DefinedStruct::PortConnectorInformation(s) => {
            register!(serial format!("/dev/smbios/port_connector/{}", pos) => s)?
        }
        DefinedStruct::ProcessorAdditionalInformation(s) => {
            register!(serial format!("/dev/smbios/processor_additional_info/{}", pos) => s)?
        }
        DefinedStruct::ProcessorInformation(s) => {
            register!(serial format!("/dev/smbios/processor_info/{}", pos) => s)?
        }
        DefinedStruct::SystemBootInformation(s) => {
            register!(serial "/dev/smbios/system_boot" => s)?
        }
        DefinedStruct::SystemChassisInformation(s) => {
            register!(serial format!("/dev/smbios/system_chassis/{}", pos) => s)?
        }
        DefinedStruct::SystemConfigurationOptions(s) => {
            register!(serial format!("/dev/smbios/system_config/{}", pos) => s)?
        }
        DefinedStruct::SystemInformation(s) => {
            register!(serial format!("/dev/smbios/system_info/{}", pos) => s)?
        }
        DefinedStruct::SystemPowerControls(s) => {
            register!(serial format!("/dev/smbios/system_power_controls/{}", pos) => s)?
        }
        DefinedStruct::SystemPowerSupply(s) => {
            register!(serial format!("/dev/smbios/system_power_supply/{}", pos) => s)?
        }
        DefinedStruct::SystemSlot(s) => {
            register!(serial format!("/dev/smbios/system_slot/{}", pos) => s)?
        }
        DefinedStruct::TemperatureProbe(s) => {
            register!(serial format!("/dev/smbios/temperature_probe/{}", pos) => s)?
        }
        DefinedStruct::TpmDevice(s) => {
            register!(serial format!("/dev/smbios/tmp_device/{}", pos) => s)?
        }
        DefinedStruct::VoltageProbe(s) => {
            register!(serial format!("/dev/smbios/voltage/{}", pos) => s)?
        }
        DefinedStruct::Undefined(s) => {
            register!(serial format!("/dev/smbios/undefined/{}", pos) => s)?
        }
        DefinedStruct::Inactive(_) => {}
        DefinedStruct::EndOfTable(_) => {}
    }
    Ok(())
}

pub fn init() {
    info!("[SMBIOS] Setting up");
    let mut mapper = MemoryMapperImpl {};
    let data = match smbioslib::table_load_from_device(&mut mapper) {
        Ok(data) => data,
        Err(e) => {
            info!("[SMBIOS] Failed with error {:?}", e);
            return;
        }
    };
    let table: DefinedStructTable<'_> = data.iter().collect();
    let iter: IntoIter<DefinedStruct<'_>> = table.into_iter();
    for values in iter
        .sorted_by_key(|v| core::intrinsics::discriminant_value(v) as u32)
        .collect_vec()
        .group_by(|a, b| discriminant(a) == discriminant(b))
    {
        for (pos, v) in values.iter().enumerate() {
            if let Err(e) = add_struct(pos, v) {
                error!("[SMBIOS] Failed to add SMBIOS structure: {:?}", e)
            }
        }
    }
    info!("[SMBIOS] Setup complete");
}
