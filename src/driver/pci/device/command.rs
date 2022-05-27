use pci_types::{ConfigRegionAccess, EndpointHeader, PciAddress, PciHeader};
use crate::flow::{FlowManagerError, VarHandler, VarProvider};
use libkernel::flow::BoolMessage;

type Result<T> = core::result::Result<T, FlowManagerError>;

macro_rules! command_var {
    ($path:expr => $name:ident get $getter:ident set $setter:ident) => {
        struct $name<T: ConfigRegionAccess + Sync + Clone + 'static> {
            access: T,
            header: PciHeader
        }

        impl<T: ConfigRegionAccess + Sync + Clone + 'static> $name<T> {
            fn register(access: &T, address: PciAddress) -> Result<()> {
                let header = PciHeader::new(address);
                let obj = $name {
                    access: access.clone(),
                    header
                };
                register!(var format!("/dev/pci/{}/{}/{}/command/{}", address.bus(), address.device(), address.function(), $path) => BoolMessage obj);
                Ok(())
            }
        }

        impl<T: ConfigRegionAccess + Sync + Clone + 'static> VarHandler<BoolMessage> for $name<T> {
            fn get(&self) -> BoolMessage {
                BoolMessage::new(self.header.command(&self.access).$getter())
            }

            fn set(&self, v: BoolMessage) {
                let command = self.header.command(&self.access);
                let command = command.builder().$setter(v.get()).build();
                self.header.set_command(command, &self.access);
            }
        }
    };
}

command_var!("interrupts_disabled" => InterruptsDisabled get interrupts_disabled set interrupt_disable);
command_var!("fast_back_to_back_enabled" => FastBackToBackEnabled get fast_back_to_back_enabled set fast_back_to_back_enable);
command_var!("serr_enabled" => SerrEnabled get serr_enabled set serr_enable);
command_var!("parity_error_response" => ParityErrorResponse get parity_error_response set parity_error_response);
command_var!("vga_palette_snoop" => VgaPaletteSnoop get vga_palette_snoop set vga_palette_snoop);
command_var!("memory_write_and_invalidate_enabled" => MemoryWriteAndInvalidateEnabled get memory_write_and_invalidate_enabled set memory_write_and_invalidate_enable);
command_var!("monitor_special_cycles" => MonitorSpecialCycles get monitor_special_cycles set monitor_special_cycles);
command_var!("bus_mastering_enabled" => BusMasteringEnabled get bus_mastering_enabled set bus_mastering);
command_var!("memory_space_access_enabled" => MemorySpaceAccessEnabled get memory_space_access_enabled set memory_space_access);
command_var!("io_space_access_enabled" => IoSpaceAccessEnabled get io_space_access_enabled set io_space_access);

pub fn register_command<T: ConfigRegionAccess + Sync + Clone + 'static>(address: PciAddress, access: &T) -> Result<()> {
    InterruptsDisabled::register(access, address)?;
    FastBackToBackEnabled::register(access, address)?;
    SerrEnabled::register(access, address)?;
    ParityErrorResponse::register(access, address)?;
    VgaPaletteSnoop::register(access, address)?;
    MemoryWriteAndInvalidateEnabled::register(access, address)?;
    MonitorSpecialCycles::register(access, address)?;
    BusMasteringEnabled::register(access, address)?;
    MemorySpaceAccessEnabled::register(access, address)?;
    IoSpaceAccessEnabled::register(access, address)?;
    Ok(())
}